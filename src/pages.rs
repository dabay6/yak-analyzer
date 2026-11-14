//! Reading the files that hold scripts inside another format: a Vue component's script blocks, an MDX
//! document's `import` and `export … from` statements, and an HTML page's script elements (spec PARSE-3
//! to PARSE-5, IMP-1).
//!
//! Each reader is the analyzer's own and small. It finds only what the analyzer needs from its format,
//! and the scripts it finds are parsed as any script is.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::imports::{self, Broken, DEFAULT, EVERY, Form, Import, Name, Read, Used};

/// The elements of an HTML page whose content is text a browser does not read as markup.
const RAW_TEXT: [&str; 9] = [
    "script", "style", "textarea", "title", "xmp", "iframe", "noembed", "noframes", "noscript",
];

/// What a reader passed over that a notice is due for.
#[derive(Debug, PartialEq, Eq)]
pub enum Skipped {
    /// A Vue script block in a language the analyzer does not read, with its line and its `lang`.
    Language { line: usize, language: String },
    /// A line of an MDX document that begins like an `import` or `export` statement and does not parse
    /// as one.
    Statement { line: usize },
}

/// The script blocks of a Vue component: each block's `src` and the imports and names of its content,
/// in the language its `lang` names; a block in another language is passed over. A block that does not
/// parse makes the component not parse, at the block's line in the file (spec PARSE-3).
///
/// The component's default export is the component, which is never reported, so it is not among its
/// exports. A name its template holds uses the export of that name, and the binding of that name that
/// an import of a `<script setup>` block makes (spec USE-4).
pub fn vue(text: &str) -> Result<(Read, Vec<Skipped>), Broken> {
    let mut read = Read::default();
    let mut skipped = Vec::new();
    let blocks = blocks(text);
    let mut named: BTreeMap<String, usize> = BTreeMap::new();

    for block in blocks.iter().filter(|block| block.tag.name == "template") {
        words(
            &text[block.content.clone()],
            line_at(text, block.content.start),
            true,
            &mut named,
        );
    }

    for block in blocks.iter().filter(|block| block.tag.name == "script") {
        let line = line_at(text, block.start);

        if let Some(src) = block.tag.attribute("src") {
            read.imports.push(source(src, line));
        }

        // The lines of the file above the block's content.
        let above = line_at(text, block.content.start) - 1;
        let lang = block.tag.attribute("lang");

        match imports::read_block(&text[block.content.clone()], lang) {
            None => skipped.push(Skipped::Language {
                line,
                language: lang.unwrap_or_default().to_owned(),
            }),
            Some(Ok(found)) => {
                let setup = block.tag.attribute("setup").is_some();

                read.imports.extend(found.imports.into_iter().map(|import| {
                    let mut import = moved(import, above);

                    if setup {
                        name_bindings(&mut import, &named);
                    }

                    import
                }));
                read.computed += found.computed;

                for export in found.exports {
                    if export.name != DEFAULT
                        && read
                            .exports
                            .iter()
                            .all(|earlier| earlier.name != export.name)
                    {
                        read.exports.push(imports::Export {
                            line: export.line + above,
                            ..export
                        });
                    }
                }
            }
            Some(Err(broken)) => {
                return Err(Broken {
                    line: broken.line.map(|line| line + above),
                    message: broken.message,
                });
            }
        }
    }

    read.named = read
        .exports
        .iter()
        .filter_map(|export| {
            named.get(&export.name).map(|line| Used {
                name: Some(export.name.clone()),
                line: *line,
            })
        })
        .collect();

    Ok((read, skipped))
}

/// An import of a block, with its lines counted in the file: the lines above the block are added.
fn moved(import: Import, above: usize) -> Import {
    Import {
        line: import.line + above,
        uses: import
            .uses
            .into_iter()
            .map(|used| Used {
                line: used.line + above,
                ..used
            })
            .collect(),
        ..import
    }
}

/// Records, for each binding of an import that a text names, the use the text makes of it, from the
/// line of the name's first place in the text, unless a script already uses it (spec USE-4, USE-5).
fn name_bindings(import: &mut Import, named: &BTreeMap<String, usize>) {
    for Name { name, local, .. } in &import.names {
        let Some(line) = local.as_ref().and_then(|local| named.get(local)) else {
            continue;
        };
        let used = (name != EVERY).then(|| name.clone());

        if import.uses.iter().all(|earlier| earlier.name != used) {
            import.uses.push(Used {
                name: used,
                line: *line,
            });
        }
    }
}

/// Adds each identifier-shaped word of a text to the words given, with the line of its first place;
/// `first` is the line the text begins on. With `tags`, a word written with hyphens is also added in
/// camelCase and in PascalCase, as a template names a component by its tag (spec USE-4). This is a
/// scan and not a compiler: it finds more names than a compiler would, never fewer.
fn words(text: &str, first: usize, tags: bool, found: &mut BTreeMap<String, usize>) {
    let is_word = |each: char| each.is_alphanumeric() || each == '_' || each == '$';
    let mut add = |word: String, line: usize| {
        if word.chars().next().is_some_and(|start| !start.is_numeric()) {
            found.entry(word).or_insert(line);
        }
    };

    for (index, line) in text.split('\n').enumerate() {
        for chain in line.split(|each: char| !(is_word(each) || each == '-')) {
            let parts: Vec<&str> = chain.split('-').filter(|part| !part.is_empty()).collect();

            for part in &parts {
                add((*part).to_owned(), first + index);
            }

            if tags && parts.len() > 1 {
                let pascal: String = parts.iter().map(|part| capitalized(part)).collect();
                let camel = parts[0].to_owned()
                    + &parts[1..]
                        .iter()
                        .map(|part| capitalized(part))
                        .collect::<String>();

                add(pascal, first + index);
                add(camel, first + index);
            }
        }
    }
}

/// A word with its first letter in upper case.
fn capitalized(word: &str) -> String {
    let mut letters = word.chars();

    letters.next().map_or_else(String::new, |start| {
        start.to_uppercase().chain(letters).collect()
    })
}

/// The `import` and `export … from` statements of an MDX document, and nothing else (spec PARSE-4). A
/// statement begins at the start of a line with `import` or `export`, outside the front matter and
/// outside a fenced code block, and runs to the next blank line. One that does not parse is text.
///
/// A binding an import makes is used where the document's text names it, outside its statements and
/// its fenced code blocks (spec USE-5).
pub fn mdx(text: &str) -> (Read, Vec<Skipped>) {
    let lines: Vec<(usize, &str)> = text
        .split_inclusive('\n')
        .scan(0, |start, line| {
            let at = *start;

            *start += line.len();

            Some((at, line.trim_end_matches(['\n', '\r'])))
        })
        .collect();
    let mut imports = Vec::new();
    let mut skipped = Vec::new();
    let mut index = front_matter(&lines);
    let mut fence: Option<(char, usize)> = None;
    let mut named: BTreeMap<String, usize> = BTreeMap::new();

    while let Some((start, line)) = lines.get(index).copied() {
        if let Some((mark, size)) = fence {
            if closes(line, mark, size) {
                fence = None;
            }

            index += 1;

            continue;
        }

        if let Some(opened) = opens(line) {
            fence = Some(opened);
            index += 1;

            continue;
        }

        if !begins_statement(line) {
            words(line, index + 1, false, &mut named);
            index += 1;

            continue;
        }

        let end = (index + 1..lines.len())
            .find(|after| lines[*after].1.trim().is_empty())
            .unwrap_or(lines.len());
        let stop = lines.get(end).map_or(text.len(), |(at, _)| *at);

        match imports::read_statements(&text[start..stop]) {
            Ok(found) => imports.extend(found.into_iter().map(|import| moved(import, index))),
            Err(_) => {
                skipped.push(Skipped::Statement { line: index + 1 });

                // A statement that does not parse is text, so its words are the text's.
                for (offset, (_, line)) in lines[index..end].iter().enumerate() {
                    words(line, index + offset + 1, false, &mut named);
                }
            }
        }

        index = end;
    }

    for import in &mut imports {
        name_bindings(import, &named);
    }

    (
        Read {
            imports,
            ..Read::default()
        },
        skipped,
    )
}

/// The `src` of every script element of an HTML page, read leniently, as a browser reads markup; nothing
/// else, and not the content of a script element (spec PARSE-5).
pub fn html(text: &str) -> Read {
    let lowered = text.to_ascii_lowercase();
    let mut imports = Vec::new();
    let mut at = 0;

    while let Some(found) = text[at..].find('<') {
        let start = at + found;
        let rest = &text[start..];

        if rest.starts_with("<!--") {
            at = past(text, start, "-->");
        } else if rest.starts_with("<!") || rest.starts_with("<?") || rest.starts_with("</") {
            at = past(text, start, ">");
        } else if let Some(tag) = open_tag(text, start, true) {
            if tag.name == "script"
                && let Some(src) = tag.attribute("src").map(str::trim)
                && !src.is_empty()
            {
                imports.push(source(&src.replace("&amp;", "&"), line_at(text, start)));
            }

            // A browser reads the content of these as text up to their end tag, whatever it holds.
            at = if RAW_TEXT.contains(&tag.name.as_str()) {
                let end = format!("</{}", tag.name);

                lowered[tag.end..]
                    .find(&end)
                    .map_or(text.len(), |found| past(text, tag.end + found, ">"))
            } else {
                tag.end
            };
        } else {
            at = start + 1;
        }
    }

    Read {
        imports,
        ..Read::default()
    }
}

/// The import a `src` attribute makes. The script it names is the block's or the page's own, so every
/// export of it is used (spec USE-2).
fn source(src: &str, line: usize) -> Import {
    Import {
        specifier: src.to_owned(),
        form: Form::Src,
        type_only: false,
        line,
        names: Vec::new(),
        uses: vec![Used { name: None, line }],
    }
}

/// The line an offset of a text lies on, counted from one.
fn line_at(text: &str, offset: usize) -> usize {
    text[..offset].matches('\n').count() + 1
}

/// The offset just past the first `end` at or after `from`, or the end of the text.
fn past(text: &str, from: usize, end: &str) -> usize {
    text[from..]
        .find(end)
        .map_or(text.len(), |found| from + found + end.len())
}

/// One start tag.
#[derive(Debug)]
struct Tag {
    /// Its name.
    name: String,
    /// Its attributes, each with its value where it has one.
    attributes: Vec<(String, Option<String>)>,
    /// The offset just past its `>`.
    end: usize,
    /// Whether it ends with `/>`.
    closed: bool,
}

impl Tag {
    /// The value of an attribute, the first of the name; an attribute with no value is empty.
    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(each, _)| each == name)
            .map(|(_, value)| value.as_deref().unwrap_or_default())
    }
}

/// The start tag at an offset that holds `<`, where one begins there: a name that begins with a letter,
/// then attributes, to `>`. With `lower`, the tag's name and its attributes' names are put in lower
/// case, as HTML reads them.
fn open_tag(text: &str, start: usize, lower: bool) -> Option<Tag> {
    let bytes = text.as_bytes();
    let named = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'-';
    let mut at = start + 1;

    if !bytes.get(at)?.is_ascii_alphabetic() {
        return None;
    }

    while bytes.get(at).copied().is_some_and(named) {
        at += 1;
    }

    let fold = |name: &str| {
        if lower {
            name.to_ascii_lowercase()
        } else {
            name.to_owned()
        }
    };
    let name = fold(&text[start + 1..at]);
    let mut attributes = Vec::new();

    loop {
        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }

        match bytes[at] {
            b'>' => {
                return Some(Tag {
                    name,
                    attributes,
                    end: at + 1,
                    closed: false,
                });
            }
            b'/' if bytes.get(at + 1) == Some(&b'>') => {
                return Some(Tag {
                    name,
                    attributes,
                    end: at + 2,
                    closed: true,
                });
            }
            b'/' | b'=' => {
                at += 1;

                continue;
            }
            _ => {}
        }

        let begins = at;

        while bytes
            .get(at)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !b"=>/".contains(byte))
        {
            at += 1;
        }

        let attribute = fold(&text[begins..at]);

        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }

        if bytes[at] != b'=' {
            attributes.push((attribute, None));

            continue;
        }

        at += 1;

        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }

        let value = match bytes[at] {
            quote @ (b'"' | b'\'') => {
                let close = text[at + 1..].find(char::from(quote))? + at + 1;
                let value = &text[at + 1..close];

                at = close + 1;
                value
            }
            _ => {
                let begins = at;

                while bytes
                    .get(at)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
                {
                    at += 1;
                }

                &text[begins..at]
            }
        };

        attributes.push((attribute, Some(value.to_owned())));
    }
}

/// One top-level block of a Vue component.
#[derive(Debug)]
struct Block {
    /// Its start tag.
    tag: Tag,
    /// The offset of its `<`.
    start: usize,
    /// Its content, between its start tag and its end tag.
    content: Range<usize>,
}

/// The top-level blocks of a Vue component, in order. A `<template>` may hold `<template>` elements;
/// every other block ends at the first end tag of its name, as Vue's own reader ends one.
fn blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut at = 0;

    while let Some(found) = text[at..].find('<') {
        let start = at + found;

        if text[start..].starts_with("<!--") {
            at = past(text, start, "-->");

            continue;
        }

        let Some(tag) = open_tag(text, start, false) else {
            at = start + 1;

            continue;
        };

        let (content, after) = if tag.closed {
            (tag.end..tag.end, tag.end)
        } else {
            closing(text, &tag.name, tag.end)
        };

        at = after;
        blocks.push(Block {
            tag,
            start,
            content,
        });
    }

    blocks
}

/// Where the content of a block that begins at `from` ends, and the offset past its end tag; the end
/// of the text when it has none. Nested blocks of the same name are counted for a `<template>` only.
fn closing(text: &str, name: &str, from: usize) -> (Range<usize>, usize) {
    let opening = format!("<{name}");
    let ending = format!("</{name}");
    let bounded = |at: usize, prefix: &str| {
        text[at..].starts_with(prefix)
            && text[at + prefix.len()..]
                .bytes()
                .next()
                .is_none_or(|byte| byte.is_ascii_whitespace() || byte == b'>' || byte == b'/')
    };
    let mut depth = 0_usize;
    let mut at = from;

    while let Some(found) = text[at..].find('<') {
        let start = at + found;

        if bounded(start, &ending) {
            if depth == 0 {
                return (from..start, past(text, start, ">"));
            }

            depth -= 1;
        } else if name == "template"
            && bounded(start, &opening)
            && open_tag(text, start, false).is_some_and(|tag| !tag.closed)
        {
            depth += 1;
        }

        at = start + 1;
    }

    (from..text.len(), text.len())
}

/// The first line after an MDX document's front matter: a block that opens with `---` on the first
/// line and closes with `---` on a later one. Without one, the first line.
fn front_matter(lines: &[(usize, &str)]) -> usize {
    if lines
        .first()
        .is_none_or(|(_, line)| line.trim_end() != "---")
    {
        return 0;
    }

    lines
        .iter()
        .skip(1)
        .position(|(_, line)| line.trim_end() == "---")
        .map_or(0, |closing| closing + 2)
}

/// The mark and length of the fence a line opens: up to three spaces, then three or more backticks or
/// tildes.
fn opens(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start_matches(' ');

    if line.len() - trimmed.len() > 3 {
        return None;
    }

    let mark = trimmed
        .chars()
        .next()
        .filter(|mark| ['`', '~'].contains(mark))?;
    let size = trimmed.chars().take_while(|each| *each == mark).count();

    (size >= 3).then_some((mark, size))
}

/// Whether a line closes a fence of the mark and length given: up to three spaces, at least as many of
/// the mark, and nothing after but spaces.
fn closes(line: &str, mark: char, size: usize) -> bool {
    let trimmed = line.trim_start_matches(' ');
    let run = trimmed.chars().take_while(|each| *each == mark).count();

    line.len() - trimmed.len() <= 3 && run >= size && trimmed[run..].trim().is_empty()
}

/// Whether a line begins like an `import` or `export` statement: the word at its very start, followed
/// by anything that cannot continue a name.
fn begins_statement(line: &str) -> bool {
    ["import", "export"].iter().any(|word| {
        line.strip_prefix(word).is_some_and(|rest| {
            rest.chars()
                .next()
                .is_none_or(|next| !(next.is_alphanumeric() || next == '_' || next == '$'))
        })
    })
}
