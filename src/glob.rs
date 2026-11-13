//! Globs: patterns for paths, in the dialect the spec defines (spec KEY-5).
//!
//! A glob is turned into a regular expression once, and matched against a path relative to the glob's
//! folder, written with forward slashes, with exact letter case.

use regex::Regex;

/// A glob, ready to match.
#[derive(Debug)]
pub struct Glob {
    /// The regular expression it stands for.
    pattern: Regex,
}

impl Glob {
    /// Reads a glob. The error says why it cannot be read.
    pub fn new(written: &str) -> Result<Self, String> {
        let expression = translate(written)?;
        let pattern = Regex::new(&format!("^{expression}$"))
            .map_err(|_| format!("the glob `{written}` cannot be read"))?;

        Ok(Self { pattern })
    }

    /// Whether the glob matches a path, relative to the glob's folder.
    pub fn matches(&self, path: &str) -> bool {
        self.pattern.is_match(path)
    }
}

/// The regular expression a glob stands for.
fn translate(glob: &str) -> Result<String, String> {
    let characters: Vec<char> = glob.chars().collect();
    let mut expression = String::with_capacity(glob.len() * 2);
    let mut braces = 0_usize;
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        match character {
            '*' if characters.get(index + 1) == Some(&'*') && whole_segment(&characters, index) => {
                let at_start = index == 0;
                let at_end = index + 2 == characters.len();

                expression.push_str(match (at_start, at_end) {
                    // `**` alone matches every path.
                    (true, true) => ".*",
                    // `**/` at the start matches any number of whole folders, none included.
                    (true, false) => "(?:[^/]+/)*",
                    // `/**` at the end matches everything under the folder before it.
                    (false, true) => ".+",
                    // `/**/` in the middle matches any number of whole folders, none included.
                    (false, false) => "(?:[^/]+/)*",
                });

                // The slash after a `**` that is not at the end is part of what was written above.
                index += if at_end { 2 } else { 3 };

                continue;
            }
            '*' => expression.push_str("[^/]*"),
            '?' => expression.push_str("[^/]"),
            '[' => {
                let (class, next) = set(&characters, index, glob)?;

                expression.push_str(&class);
                index = next;

                continue;
            }
            '{' => {
                braces += 1;
                expression.push_str("(?:");
            }
            ',' if braces > 0 => expression.push('|'),
            '}' if braces > 0 => {
                braces -= 1;
                expression.push(')');
            }
            '\\' => {
                let escaped = characters.get(index + 1).ok_or_else(|| {
                    format!("the glob `{glob}` ends in a backslash that escapes nothing")
                })?;

                expression.push_str(&regex::escape(&escaped.to_string()));
                index += 2;

                continue;
            }
            other => expression.push_str(&regex::escape(&other.to_string())),
        }

        index += 1;
    }

    if braces > 0 {
        return Err(format!("the glob `{glob}` has a brace left open"));
    }

    Ok(expression)
}

/// Whether the `**` at the index is a whole path segment: at the start or after a slash, and at the
/// end or before a slash. Anywhere else it is two `*`.
fn whole_segment(characters: &[char], index: usize) -> bool {
    let before = index == 0 || characters[index - 1] == '/';
    let after = index + 2 == characters.len() || characters[index + 2] == '/';

    before && after
}

/// The character class a `[` at the index begins, and the index after its `]`. A `!` first means any
/// character not in the set, a `]` first is a character of the set, and a class never matches `/`.
fn set(characters: &[char], start: usize, glob: &str) -> Result<(String, usize), String> {
    let mut index = start + 1;
    let mut class = String::from("[[");

    if characters.get(index) == Some(&'!') {
        class.push('^');
        index += 1;
    }

    let first = index;

    loop {
        let character = *characters
            .get(index)
            .ok_or_else(|| format!("the glob `{glob}` has a bracket left open"))?;

        if character == ']' && index > first {
            break;
        }

        match character {
            '\\' => {
                let escaped = *characters
                    .get(index + 1)
                    .ok_or_else(|| format!("the glob `{glob}` has a bracket left open"))?;

                if escaped.is_ascii_punctuation() {
                    class.push('\\');
                }

                class.push(escaped);
                index += 2;

                continue;
            }
            // A dash between two characters is a range; anywhere else it is itself.
            '-' if index > first && characters.get(index + 1).is_some_and(|next| *next != ']') => {
                class.push('-');
            }
            other if other.is_ascii_punctuation() => {
                class.push('\\');
                class.push(other);
            }
            other => class.push(other),
        }

        index += 1;
    }

    // The set, negated or not, intersected with every character but `/`, so that a class stays inside
    // one path segment.
    class.push_str("]&&[^/]]");

    Ok((class, index + 1))
}

#[cfg(test)]
mod tests {
    use super::Glob;

    /// Whether the glob matches each path given, as the spec's dialect says it must.
    fn check(written: &str, matching: &[&str], not_matching: &[&str]) {
        let glob = Glob::new(written).expect("the glob can be read");

        for path in matching {
            assert!(glob.matches(path), "`{written}` should match `{path}`");
        }

        for path in not_matching {
            assert!(!glob.matches(path), "`{written}` should not match `{path}`");
        }
    }

    #[test]
    fn a_star_stays_inside_one_segment() {
        check(
            "src/*.ts",
            &["src/a.ts", "src/.ts"],
            &["src/a/b.ts", "a.ts", "src/a.tsx"],
        );
    }

    #[test]
    fn a_double_star_spans_whole_segments() {
        check(
            "**/fixtures/**",
            &["fixtures/a.json", "a/b/fixtures/c/d.ts"],
            &["fixtures", "a/myfixtures/b.ts"],
        );
        check(
            "src/**/x.ts",
            &["src/x.ts", "src/a/b/x.ts"],
            &["srcx.ts", "src/ax.ts"],
        );
        check("**", &["a", "a/b/c.ts"], &[]);
        check(
            "folder/**",
            &["folder/a.ts", "folder/a/b.ts"],
            &["folder", "other/a.ts"],
        );
        check("a**b.ts", &["ab.ts", "axxb.ts"], &["a/b.ts"]);
    }

    #[test]
    fn a_question_mark_is_one_character_and_never_a_slash() {
        check("?.ts", &["a.ts", "é.ts"], &["ab.ts", "/.ts"]);
    }

    #[test]
    fn a_set_may_hold_a_range_be_negated_and_begin_with_a_bracket() {
        check("[a-c].ts", &["a.ts", "c.ts"], &["d.ts", "A.ts"]);
        check("[!_]*.mjs", &["run.mjs"], &["_private.mjs"]);
        check("[]x].ts", &["].ts", "x.ts"], &["y.ts"]);
        check("[-a].ts", &["-.ts", "a.ts"], &["b.ts"]);
        check("x[!a]y", &["xby"], &["xay", "x/y"]);
    }

    #[test]
    fn braces_hold_alternatives_and_may_nest() {
        check(
            "**/*.{test,spec}.{ts,tsx}",
            &["a.test.ts", "b/c.spec.tsx"],
            &["a.test.js", "a.tests.ts"],
        );
        check(
            "docs/{a,{b,c}}/*.mdx",
            &["docs/a/x.mdx", "docs/c/y.mdx"],
            &["docs/d/x.mdx"],
        );
    }

    #[test]
    fn a_backslash_makes_a_character_stand_for_itself_and_the_rest_stand_for_themselves() {
        check("odd\\[name\\]/?.ts", &["odd[name]/a.ts"], &["oddn/a.ts"]);
        check("a+b(c).ts", &["a+b(c).ts"], &["aab(c).ts"]);
        check("Src/A.ts", &["Src/A.ts"], &["src/a.ts"]);
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        for (glob, why) in [
            ("src/[a-z.ts", "has a bracket left open"),
            ("src/{a,b.ts", "has a brace left open"),
            ("src/a\\", "ends in a backslash that escapes nothing"),
            ("[]", "has a bracket left open"),
        ] {
            let refused = Glob::new(glob).expect_err(glob);

            assert!(refused.ends_with(why), "{refused}");
        }
    }
}
