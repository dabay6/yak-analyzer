//! The files a CI system reads commands from: the command texts and folders a declaration's property
//! paths reach in one, over the values `yaml` reads (comparison spec CI-1, CI-6).

use crate::yaml::Node;

/// Each command text a property path reaches in a document, with the mappings it lies under, the one
/// that holds it last. `*` stands for every key of a mapping and every item of a sequence, and a
/// sequence of texts at the path's end is a command text each; every scalar is a text (comparison spec
/// CI-1, CI-6).
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
        if let Some(text) = node.text() {
            found.push((text, above.clone()));
        } else if let Node::List(items) = node {
            for item in items {
                if let Some(text) = item.text() {
                    found.push((text, above.clone()));
                }
            }
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
        return node.text();
    };

    match node {
        Node::Map(entries) => entries
            .iter()
            .filter(|(name, _)| name == key)
            .find_map(|(_, value)| text_at(value, rest)),
        _ => None,
    }
}
