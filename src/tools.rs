//! What a tool starts, read without running anything: the commands of a `package.json`'s scripts, split
//! into words as far as ENTRY-7 reads a shell, and a literal value of a tool's configuration file
//! (spec ENTRY-6, ENTRY-7, design AN-6).

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrayExpressionElement, AssignmentTarget, Expression, ObjectExpression, ObjectPropertyKind,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// The text between unquoted separators that ends one command and begins the next (spec ENTRY-7).
const SEPARATORS: [&str; 5] = ["&&", "||", ";", "|", "&"];

/// The commands of a script, each as its words, quotes removed. Quotes are found first, from the left,
/// and nothing quoted splits; then the script is cut into commands at unquoted `&&`, `||`, `;`, `|` and
/// `&`, and each command into words at unquoted white space (spec ENTRY-7).
pub fn commands(script: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    let mut rest = script;

    while let Some(character) = rest.chars().next() {
        if let Some(open) = quote {
            if character == open {
                quote = None;
            } else {
                word.push(character);
            }

            rest = &rest[character.len_utf8()..];
            continue;
        }

        if let Some(separator) = SEPARATORS
            .iter()
            .find(|separator| rest.starts_with(**separator))
        {
            end_word(&mut words, &mut word, &mut started);
            commands.push(std::mem::take(&mut words));
            rest = &rest[separator.len()..];
            continue;
        }

        if character.is_whitespace() {
            end_word(&mut words, &mut word, &mut started);
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            started = true;
        } else {
            word.push(character);
            started = true;
        }

        rest = &rest[character.len_utf8()..];
    }

    end_word(&mut words, &mut word, &mut started);
    commands.push(words);
    commands.retain(|command| !command.is_empty());
    commands
}

/// Ends the word being read, when one was begun: a pair of quotes alone makes an empty word.
fn end_word(words: &mut Vec<String>, word: &mut String, started: &mut bool) {
    if *started {
        words.push(std::mem::take(word));
        *started = false;
    }
}

/// Whether a word of a command is an assignment that comes before its program, `NAME=value`.
pub fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        name.starts_with(|first: char| first.is_ascii_alphabetic() || first == '_')
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
    })
}

/// The value a configuration file states at a property path, where it is written as a string or a list
/// of strings in the object that is the file's default export, or in the one object passed to the call
/// that is its default export, reached through the path by object literals alone. Nothing where the file
/// does not parse, or the value is written any other way: the file is never run (spec ENTRY-6, design
/// AN-6).
pub fn configured(path: &str, text: &str, property: &[String]) -> Option<Vec<String>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type(path)).parse();

    if parsed.fatal_error || !parsed.diagnostics.is_empty() {
        return None;
    }

    let exported = parsed.program.body.iter().rev().find_map(default_export)?;
    let mut object = object_of(exported)?;
    let (last, path) = property.split_last()?;

    for key in path {
        object = match unwrapped(value_of(object, key)?) {
            Expression::ObjectExpression(inner) => inner,
            _ => return None,
        };
    }

    strings(unwrapped(value_of(object, last)?))
}

/// The language a configuration file is parsed in, from its name: TypeScript for a TypeScript ending,
/// and otherwise JavaScript, as a module.
fn source_type(path: &str) -> SourceType {
    let typescript = [".ts", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending));

    if typescript {
        SourceType::ts()
    } else {
        SourceType::jsx()
    }
    .with_module(true)
}

/// The expression a statement exports as the default: `export default …`, or `module.exports = …`.
fn default_export<'s, 'a>(statement: &'s Statement<'a>) -> Option<&'s Expression<'a>> {
    match statement {
        Statement::ExportDefaultDeclaration(declaration) => declaration.declaration.as_expression(),
        Statement::ExpressionStatement(statement) => match &statement.expression {
            Expression::AssignmentExpression(assignment) => match &assignment.left {
                AssignmentTarget::StaticMemberExpression(member)
                    if member.property.name == "exports"
                        && matches!(&member.object, Expression::Identifier(module) if module.name == "module") =>
                {
                    Some(&assignment.right)
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// The object an exported expression is, or the one object passed to the call it is.
fn object_of<'s, 'a>(exported: &'s Expression<'a>) -> Option<&'s ObjectExpression<'a>> {
    match unwrapped(exported) {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) if call.arguments.len() == 1 => {
            match call.arguments[0].as_expression().map(unwrapped) {
                Some(Expression::ObjectExpression(object)) => Some(object),
                _ => None,
            }
        }
        _ => None,
    }
}

/// An expression without the parentheses, `as` and `satisfies` around it, which change no value.
fn unwrapped<'s, 'a>(expression: &'s Expression<'a>) -> &'s Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => unwrapped(&inner.expression),
        Expression::TSAsExpression(inner) => unwrapped(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrapped(&inner.expression),
        other => other,
    }
}

/// The value an object literal states for a key, written as a plain property; the last such one, as the
/// object's own value is the last one written.
fn value_of<'s, 'a>(object: &'s ObjectExpression<'a>, key: &str) -> Option<&'s Expression<'a>> {
    object
        .properties
        .iter()
        .rev()
        .find_map(|property| match property {
            ObjectPropertyKind::ObjectProperty(property)
                if !property.computed
                    && property
                        .key
                        .static_name()
                        .is_some_and(|name| name.as_str() == Some(key)) =>
            {
                Some(&property.value)
            }
            _ => None,
        })
}

/// The strings a value is written as: one string, or a list of strings and nothing else.
fn strings(value: &Expression<'_>) -> Option<Vec<String>> {
    match value {
        Expression::StringLiteral(literal) => Some(vec![literal.value.display().to_string()]),
        Expression::TemplateLiteral(literal) => {
            Some(vec![literal.single_quasi()?.display().to_string()])
        }
        Expression::ArrayExpression(list) => list
            .elements
            .iter()
            .map(|element| match element {
                ArrayExpressionElement::StringLiteral(literal) => {
                    Some(literal.value.display().to_string())
                }
                ArrayExpressionElement::TemplateLiteral(literal) => {
                    Some(literal.single_quasi()?.display().to_string())
                }
                _ => None,
            })
            .collect(),
        _ => None,
    }
}
