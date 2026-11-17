//! The files a CI system reads commands from: each read as YAML within fixed bounds, and the command
//! texts and folders a declaration's property paths reach in it (comparison spec CI-1 to CI-3).

use std::collections::BTreeMap;

use yaml_rust2::parser::{Event, EventReceiver, Parser};

/// The largest command file the analyzer reads, in bytes (comparison spec CI-3).
pub const MAX_BYTES: usize = 1024 * 1024;

/// The most values a command file may hold, all its documents together, once its aliases are followed
/// (comparison spec CI-3).
const MAX_VALUES: usize = 100_000;

/// The most text, in bytes, a command file may hold, all its documents together, once its aliases are
/// followed: an alias of one long text copies all of it, however few values it counts (comparison spec
/// CI-3).
const MAX_TEXT: usize = 4 * MAX_BYTES;

/// One value of a command file.
#[derive(Clone, Debug)]
pub enum Node {
    /// A text: every scalar, as written.
    Text(String),
    /// A sequence.
    List(Vec<Self>),
    /// A mapping, by its keys that are texts; a key of another kind is passed over with its value.
    Map(Vec<(String, Self)>),
}

/// Why a command file adds nothing (comparison spec CI-3).
#[derive(Debug)]
pub enum Unread {
    /// It is not valid YAML, or not UTF-8.
    Invalid,
    /// It is over 1 MiB, or holds more than 100,000 values or 4 MiB of text, all its documents
    /// together, once its aliases are followed.
    TooLarge,
}

/// Every document of a command file's text. Values and their text are counted as the parser hands them
/// over, across all the file's documents, and an alias counts the values and text of the node it names
/// each time it appears, so reading stops at the bounds before any copy is made (comparison spec CI-3).
pub fn read(text: &str) -> Result<Vec<Node>, Unread> {
    let mut builder = Builder::default();

    Parser::new_from_str(text)
        .load(&mut builder, true)
        .map_err(|_| Unread::Invalid)?;

    if builder.over {
        Err(Unread::TooLarge)
    } else {
        Ok(builder.documents)
    }
}

/// Each command text a property path reaches in a document, with the mappings it lies under, the one
/// that holds it last. `*` stands for every key of a mapping and every item of a sequence, and a
/// sequence of texts at the path's end is a command text each (comparison spec CI-1).
pub fn reached<'n>(document: &'n Node, path: &[String]) -> Vec<(&'n str, Vec<&'n Node>)> {
    let mut found = Vec::new();

    walk(document, path, &mut Vec::new(), &mut found);
    found
}

/// The folder text a command runs in: each path tried from the mapping that holds the command, then
/// from each mapping above it, nearest first, every path at one mapping before the next one up; the
/// first text found. A value of another kind is passed over (comparison spec CI-1).
pub fn folder<'n>(above: &[&'n Node], paths: &[Vec<String>]) -> Option<&'n str> {
    above
        .iter()
        .rev()
        .find_map(|mapping| paths.iter().find_map(|path| text_at(mapping, path)))
}

/// Walks a property path down from a node, keeping the mappings passed through.
fn walk<'n>(
    node: &'n Node,
    path: &[String],
    above: &mut Vec<&'n Node>,
    found: &mut Vec<(&'n str, Vec<&'n Node>)>,
) {
    let Some((key, rest)) = path.split_first() else {
        match node {
            Node::Text(text) => found.push((text.as_str(), above.clone())),
            Node::List(items) => {
                for item in items {
                    if let Node::Text(text) = item {
                        found.push((text.as_str(), above.clone()));
                    }
                }
            }
            Node::Map(_) => {}
        }

        return;
    };

    match node {
        Node::Map(entries) => {
            above.push(node);

            for (name, value) in entries {
                if key == "*" || name == key {
                    walk(value, rest, above, found);
                }
            }

            above.pop();
        }
        Node::List(items) if key == "*" => {
            for item in items {
                walk(item, rest, above, found);
            }
        }
        _ => {}
    }
}

/// The text a property path reaches from a node, where it reaches one.
fn text_at<'n>(node: &'n Node, path: &[String]) -> Option<&'n str> {
    let Some((key, rest)) = path.split_first() else {
        return match node {
            Node::Text(text) => Some(text.as_str()),
            _ => None,
        };
    };

    match node {
        Node::Map(entries) => entries
            .iter()
            .filter(|(name, _)| name == key)
            .find_map(|(_, value)| text_at(value, rest)),
        _ => None,
    }
}

/// A sequence or a mapping being built.
enum Open {
    /// A sequence, with its items so far.
    List(Vec<Node>),
    /// A mapping, with its entries so far and the key waiting for its value: a text, or none for a key
    /// of another kind, whose value is passed over.
    Map {
        entries: Vec<(String, Node)>,
        key: Option<Option<String>>,
    },
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

/// A sequence or a mapping being built, with its anchor and the weight it holds so far, itself
/// included.
struct Frame {
    open: Open,
    anchor: usize,
    size: Weight,
}

/// Builds the documents from the parser's events, counting what they weigh as they come.
#[derive(Default)]
struct Builder {
    documents: Vec<Node>,
    frames: Vec<Frame>,
    anchors: BTreeMap<usize, (Node, Weight)>,
    counted: Weight,
    over: bool,
}

impl Builder {
    /// Counts a weight against the bounds, across the whole file; false once either is passed, after
    /// which nothing more is built.
    fn count(&mut self, weight: Weight) -> bool {
        self.counted = self.counted.plus(weight);
        self.over = self.counted.values > MAX_VALUES || self.counted.bytes > MAX_TEXT;

        !self.over
    }

    /// Puts a finished value where it belongs: under its anchor, then in the sequence or mapping that
    /// holds it, or as a document.
    fn finish(&mut self, node: Node, size: Weight, anchor: usize) {
        if anchor > 0 {
            self.anchors.insert(anchor, (node.clone(), size));
        }

        let Some(frame) = self.frames.last_mut() else {
            self.documents.push(node);
            return;
        };

        frame.size = frame.size.plus(size);

        match &mut frame.open {
            Open::List(items) => items.push(node),
            Open::Map { entries, key } => match key.take() {
                None => {
                    *key = Some(match node {
                        Node::Text(text) => Some(text),
                        _ => None,
                    });
                }
                Some(Some(name)) => entries.push((name, node)),
                Some(None) => {}
            },
        }
    }
}

impl EventReceiver for Builder {
    fn on_event(&mut self, event: Event) {
        if self.over {
            return;
        }

        match event {
            Event::DocumentStart => self.anchors.clear(),
            Event::Scalar(text, _, anchor, _) => {
                let weight = Weight {
                    values: 1,
                    bytes: text.len(),
                };

                if self.count(weight) {
                    self.finish(Node::Text(text), weight, anchor);
                }
            }
            Event::SequenceStart(anchor, _) => {
                if self.count(Weight::CONTAINER) {
                    self.frames.push(Frame {
                        open: Open::List(Vec::new()),
                        anchor,
                        size: Weight::CONTAINER,
                    });
                }
            }
            Event::MappingStart(anchor, _) => {
                if self.count(Weight::CONTAINER) {
                    self.frames.push(Frame {
                        open: Open::Map {
                            entries: Vec::new(),
                            key: None,
                        },
                        anchor,
                        size: Weight::CONTAINER,
                    });
                }
            }
            Event::SequenceEnd | Event::MappingEnd => {
                if let Some(frame) = self.frames.pop() {
                    let node = match frame.open {
                        Open::List(items) => Node::List(items),
                        Open::Map { entries, .. } => Node::Map(entries),
                    };

                    self.finish(node, frame.size, frame.anchor);
                }
            }
            Event::Alias(anchor) => {
                let size = self.anchors.get(&anchor).map(|(_, size)| *size);

                if let Some(size) = size
                    && self.count(size)
                {
                    let node = self.anchors[&anchor].0.clone();

                    self.finish(node, size, 0);
                }
            }
            _ => {}
        }
    }
}
