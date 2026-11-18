//! The YAML files the analyzer reads, the root's `pnpm-workspace.yaml` and the files a CI system reads
//! commands from: each read one parser event at a time, within bounds kept while it is read, with an
//! anchored value shared by its aliases rather than copied (comparison spec CI-6, CI-7).

use std::collections::BTreeMap;
use std::rc::Rc;

use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

/// The largest YAML file the analyzer reads, in bytes (comparison spec CI-3, CI-6).
pub const MAX_BYTES: usize = 1024 * 1024;

/// The most values a file may hold, all its documents together, once its aliases are followed
/// (comparison spec CI-6).
const MAX_VALUES: usize = 100_000;

/// The most text, in bytes, a file may hold, all its documents together, once its aliases are followed
/// (comparison spec CI-6).
const MAX_TEXT: usize = 4 * MAX_BYTES;

/// The deepest a file may nest once its aliases are followed, the top value at level 1 (comparison
/// spec CI-6).
const MAX_LEVELS: usize = 1_000;

/// One value of a YAML file.
#[derive(Debug)]
pub enum Node {
    /// A scalar written plain, which YAML may read as a null, a boolean or a number.
    Plain(String),
    /// A scalar written in quotes or as a block: always a text.
    Quoted(String),
    /// A sequence.
    List(Vec<Rc<Self>>),
    /// A mapping, by its keys that are scalars; a key of another kind is passed over with its value.
    Map(Vec<(String, Rc<Self>)>),
}

impl Node {
    /// The scalar's text, however it was written; none for a sequence or a mapping.
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Plain(text) | Self::Quoted(text) => Some(text),
            Self::List(_) | Self::Map(_) => None,
        }
    }
}

/// Why a YAML file was not read (comparison spec CI-3, CI-6).
#[derive(Debug)]
pub enum Unread {
    /// It is not valid YAML, for the reason given.
    Invalid(String),
    /// It is over 1 MiB, or holds more than 100,000 values, 4 MiB of text or 1,000 levels once its
    /// aliases are followed.
    TooLarge,
    /// It holds an alias inside the value it names, which has no end.
    SelfAlias,
}

/// Every document of a YAML file's text, taken one parser event at a time and never through the
/// recursive loader. Values, their text and their levels are counted as the events come, an alias
/// counting those of the node it names where it stands, and reading stops at the first event past a
/// bound, so a later error never hides it (comparison spec CI-6).
pub fn read(text: &str) -> Result<Vec<Rc<Node>>, Unread> {
    if text.len() > MAX_BYTES {
        return Err(Unread::TooLarge);
    }

    let mut parser = Parser::new_from_str(text);
    let mut builder = Builder::default();

    loop {
        let (event, _) = parser
            .next_token()
            .map_err(|error| Unread::Invalid(error.to_string()))?;

        if event == Event::StreamEnd {
            return Ok(builder.documents);
        }

        builder.take(event)?;
    }
}

/// What a value weighs against the bounds: the values it holds, itself included, and the bytes of the
/// texts among them, keys included.
#[derive(Clone, Copy, Default)]
struct Weight {
    values: usize,
    bytes: usize,
}

impl Weight {
    /// A sequence or a mapping alone, before what it holds.
    const CONTAINER: Self = Self {
        values: 1,
        bytes: 0,
    };

    /// The two weights together.
    const fn plus(self, other: Self) -> Self {
        Self {
            values: self.values.saturating_add(other.values),
            bytes: self.bytes.saturating_add(other.bytes),
        }
    }
}

/// A sequence or a mapping being built.
enum Open {
    /// A sequence, with its items so far.
    List(Vec<Rc<Node>>),
    /// A mapping, with its entries so far and the key waiting for its value: a text, or none for a key
    /// of another kind, whose value is passed over.
    Map {
        entries: Vec<(String, Rc<Node>)>,
        key: Option<Option<String>>,
    },
}

/// A sequence or a mapping being built, with its anchor, the weight it holds so far, itself included,
/// and the most levels any value it holds has.
struct Frame {
    open: Open,
    anchor: usize,
    size: Weight,
    levels: usize,
}

/// Builds the documents from the parser's events.
#[derive(Default)]
struct Builder {
    documents: Vec<Rc<Node>>,
    frames: Vec<Frame>,
    anchors: BTreeMap<usize, (Rc<Node>, Weight, usize)>,
    counted: Weight,
}

impl Builder {
    /// Takes one event: a value begun, ended or named by an alias.
    fn take(&mut self, event: Event) -> Result<(), Unread> {
        match event {
            Event::DocumentStart => self.anchors.clear(),
            Event::Scalar(text, style, anchor, _) => {
                let weight = Weight {
                    values: 1,
                    bytes: text.len(),
                };

                self.count(weight)?;

                let node = if style == TScalarStyle::Plain {
                    Node::Plain(text)
                } else {
                    Node::Quoted(text)
                };

                self.finish(Rc::new(node), weight, 0, anchor);
            }
            Event::SequenceStart(anchor, _) => self.open(Open::List(Vec::new()), anchor)?,
            Event::MappingStart(anchor, _) => self.open(
                Open::Map {
                    entries: Vec::new(),
                    key: None,
                },
                anchor,
            )?,
            Event::SequenceEnd | Event::MappingEnd => {
                if let Some(frame) = self.frames.pop() {
                    let node = match frame.open {
                        Open::List(items) => Node::List(items),
                        Open::Map { entries, .. } => Node::Map(entries),
                    };

                    self.finish(Rc::new(node), frame.size, frame.levels + 1, frame.anchor);
                }
            }
            Event::Alias(anchor) => {
                let Some((node, size, levels)) = self
                    .anchors
                    .get(&anchor)
                    .map(|(node, size, levels)| (Rc::clone(node), *size, *levels))
                else {
                    return Err(if self.frames.iter().any(|frame| frame.anchor == anchor) {
                        Unread::SelfAlias
                    } else {
                        Unread::Invalid("an alias names no anchor of its document".to_owned())
                    });
                };

                if self.frames.len() + levels > MAX_LEVELS {
                    return Err(Unread::TooLarge);
                }

                self.count(size)?;
                self.finish(node, size, levels, 0);
            }
            _ => {}
        }

        Ok(())
    }

    /// Begins a sequence or a mapping, a level deeper than the one it lies in.
    fn open(&mut self, open: Open, anchor: usize) -> Result<(), Unread> {
        if self.frames.len() + 1 > MAX_LEVELS {
            return Err(Unread::TooLarge);
        }

        self.count(Weight::CONTAINER)?;
        self.frames.push(Frame {
            open,
            anchor,
            size: Weight::CONTAINER,
            levels: 0,
        });

        Ok(())
    }

    /// Counts a weight against the bounds, across the whole file.
    fn count(&mut self, weight: Weight) -> Result<(), Unread> {
        self.counted = self.counted.plus(weight);

        if self.counted.values > MAX_VALUES || self.counted.bytes > MAX_TEXT {
            Err(Unread::TooLarge)
        } else {
            Ok(())
        }
    }

    /// Puts a finished value where it belongs: under its anchor, shared, then in the sequence or
    /// mapping that holds it, or as a document.
    fn finish(&mut self, node: Rc<Node>, size: Weight, levels: usize, anchor: usize) {
        if anchor > 0 {
            self.anchors
                .insert(anchor, (Rc::clone(&node), size, levels));
        }

        let Some(frame) = self.frames.last_mut() else {
            self.documents.push(node);
            return;
        };

        frame.size = frame.size.plus(size);
        frame.levels = frame.levels.max(levels);

        match &mut frame.open {
            Open::List(items) => items.push(node),
            Open::Map { entries, key } => match key.take() {
                None => *key = Some(node.text().map(str::to_owned)),
                Some(Some(name)) => entries.push((name, node)),
                Some(None) => {}
            },
        }
    }
}
