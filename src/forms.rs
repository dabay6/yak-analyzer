//! The statements duplication compares, read in the parse that finds a script's imports: each statement
//! list, and for each of its statements the digest of its form, its size, its place, and whether it can
//! join a run (duplication spec COPY-1, COPY-2, COPY-5, SAME-1, SAME-2, SIZE-1, DUP-5).
//!
//! A statement's form is written as the walk meets its syntax tree: the name of each node, and each
//! value that makes two nodes of one kind differ. Formatting, comments and quotes are not in the tree,
//! and parentheses that change nothing are passed over, so they leave the form as it is. What the lens
//! loosens is written as a placeholder.

use std::collections::BTreeSet;
use std::fmt;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression, ClassElement,
    JSXAttributeValue, Program, PropertyKey, Statement, TemplateElement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::{Kind, Token};
use oxc_span::{GetSpan, Span};
use sha2::{Digest, Sha256};

use crate::duplication::Lens;
use crate::functions;
use crate::imports::Lines;

/// How many bytes of the SHA-256 digest of a statement's form it keeps.
pub const DIGEST: usize = 16;

/// What ends a node in a form.
const CLOSE: u8 = b')';

/// One statement list: a file's top level, a block's body, a `switch` case's statements or a class's
/// body (spec COPY-1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct List {
    /// Its statements, in source order.
    pub items: Vec<Item>,
}

/// The lines of a statement that a token its size counts starts on (spec SIZE-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    /// The first such line, counted from one.
    pub first: usize,
    /// The last such line.
    pub last: usize,
    /// How many such lines there are.
    pub count: u64,
}

/// One statement of a list, or one member of a class's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// The first bytes of the SHA-256 digest of its form, with what the lens loosens as placeholders
    /// (spec SAME-1, SAME-2).
    pub digest: [u8; DIGEST],
    /// Whether it can join a run: an import cannot, nor a statement that holds a test callee's call
    /// (spec COPY-2, COPY-3).
    pub joins: bool,
    /// How many tokens it holds, as its tree written out in the fixed form counts them (spec SIZE-1).
    pub tokens: u64,
    /// The lines those tokens start on; none where it holds none.
    pub lines: Option<Held>,
    /// Its first byte in the file.
    pub start: u32,
    /// The byte after its last.
    pub end: u32,
    /// The line it starts on, counted from one.
    pub line: usize,
    /// The line it ends on.
    pub end_line: usize,
}

/// The statement lists of a parsed program, in the order the walk meets them, each statement read
/// through the lens, with its size counted on the program's tokens (spec COPY-5).
pub fn lists(
    program: &Program<'_>,
    tokens: &[Token],
    text: &str,
    lines: &Lines<'_>,
    lens: &Lens,
) -> Vec<List> {
    let mut former = Former {
        lens,
        stream: Vec::new(),
        lists: Vec::new(),
        open: Vec::new(),
        chained: false,
        attribute: false,
        dropped: Dropped::default(),
    };

    former.visit_program(program);

    let counted = Counted::of(tokens, text, lines, &former.dropped);

    former
        .lists
        .iter()
        .map(|raws| List {
            items: raws
                .iter()
                .map(|raw| counted.item(raw, &former.stream, lines))
                .collect(),
        })
        .collect()
}

/// A Vue component's block's lists added to the component's, with their lines and bytes counted in the
/// file: the lines above the block and the bytes before it are added.
pub fn append(lists: &mut Vec<List>, block: Vec<List>, above: usize, offset: u32) {
    lists.extend(block.into_iter().map(|list| {
        List {
            items: list
                .items
                .into_iter()
                .map(|item| Item {
                    lines: item.lines.map(|held| Held {
                        first: held.first + above,
                        last: held.last + above,
                        ..held
                    }),
                    start: item.start + offset,
                    end: item.end + offset,
                    line: item.line + above,
                    end_line: item.end_line + above,
                    ..item
                })
                .collect(),
        }
    }));
}

/// A JSX text as it renders, which is what is compared: each line trimmed of the white space at its
/// ends, the lines that hold nothing else left out, and the rest joined by one space (spec SAME-1,
/// DUP-5).
fn rendered(text: &str) -> String {
    text.split(['\n', '\r'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A text with every line ending a line feed, so a checkout's line endings change no form (spec DUP-5).
fn fed(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// A template piece's value: its cooked value, or its raw text where an invalid escape in a tagged
/// template leaves it none, with every line ending a line feed (spec SAME-1, DUP-5).
fn template(element: &TemplateElement<'_>) -> String {
    fed(&element.value.cooked.map_or_else(
        || element.value.raw.as_str().to_owned(),
        |cooked| cooked.display().to_string(),
    ))
}

/// The tokens the fixed form leaves out that only the tree tells apart (spec SIZE-1).
#[derive(Default)]
struct Dropped {
    /// Where an opening parenthesis around an expression or a type starts.
    opens: BTreeSet<u32>,
    /// Where the closing one ends.
    closes: BTreeSet<u32>,
    /// Where a `new` with no arguments ends, at its empty `()` where it has one.
    empty: BTreeSet<u32>,
    /// Where an arrow function's lone, plain parameter starts, which parentheses may wrap.
    lone: BTreeSet<u32>,
}

/// One statement as the walk found it.
struct Raw {
    /// Where its form starts in the stream.
    from: usize,
    /// Where it ends.
    to: usize,
    /// Where the statement is in the text.
    span: Span,
    /// Whether it can join a run.
    joins: bool,
}

/// Walks a program, writing the form of everything in it to one stream and noting where each
/// statement's form is.
struct Former<'l> {
    /// What to loosen.
    lens: &'l Lens,
    /// The forms, end to end.
    stream: Vec<u8>,
    /// Each list's statements, in the order the walk met the lists.
    lists: Vec<Vec<Raw>>,
    /// The statements the walk is inside, by their list and their place in it.
    open: Vec<(usize, usize)>,
    /// Whether the walk is in the callee chain of a test callee's call, where each call is part of the
    /// test's (spec COPY-3).
    chained: bool,
    /// Whether the walk is at a JSX attribute's string value.
    attribute: bool,
    /// The tokens the size leaves out.
    dropped: Dropped,
}

impl Former<'_> {
    /// Walks one statement of a list, noting where its form starts and ends in the stream.
    fn item(&mut self, list: usize, span: Span, joins: bool, walk: impl FnOnce(&mut Self)) {
        let index = self.lists[list].len();

        self.lists[list].push(Raw {
            from: self.stream.len(),
            to: self.stream.len(),
            span,
            joins,
        });
        self.open.push((list, index));
        walk(self);
        self.open.pop();
        self.lists[list][index].to = self.stream.len();
    }

    /// Writes a piece of a form: its tag, then its bytes after their length.
    fn put(&mut self, tag: u8, bytes: &[u8]) {
        self.stream.push(tag);
        self.stream
            .extend_from_slice(&u32::try_from(bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
        self.stream.extend_from_slice(bytes);
    }

    /// Writes a name, or its placeholder where names are loosened (spec SAME-2).
    fn name(&mut self, name: &str) {
        if self.lens.names {
            self.stream.push(b'n');
        } else {
            self.put(b'N', name.as_bytes());
        }
    }

    /// Writes a string's value, or its placeholder where strings are loosened (spec SAME-2).
    fn string(&mut self, value: &str) {
        if self.lens.strings {
            self.stream.push(b's');
        } else {
            self.put(b'S', value.as_bytes());
        }
    }

    /// Writes a number's value, or its placeholder where numbers are loosened (spec SAME-2).
    fn number(&mut self, value: &[u8]) {
        if self.lens.numbers {
            self.stream.push(b'd');
        } else {
            self.put(b'D', value);
        }
    }

    /// Writes the values of a node that are neither names, strings nor numbers.
    fn flags(&mut self, values: fmt::Arguments<'_>) {
        self.put(b'f', values.to_string().as_bytes());
    }

    /// Notes the parentheses of a parenthesized expression or type, which the size leaves out.
    fn unwrapped(&mut self, span: Span) {
        self.dropped.opens.insert(span.start);
        self.dropped.closes.insert(span.end);
    }

    /// Notes an arrow function's lone, plain parameter, whose parentheses the size leaves out: one with
    /// no type, default, decorator or `?`, in a function with no type parameters and no return type.
    fn bare(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        let params = &arrow.params;

        if let [only] = &params.items[..]
            && params.rest.is_none()
            && arrow.type_parameters.is_none()
            && arrow.return_type.is_none()
            && matches!(only.pattern, BindingPattern::BindingIdentifier(_))
            && only.type_annotation.is_none()
            && only.initializer.is_none()
            && only.decorators.is_empty()
            && !only.optional
        {
            self.dropped.lone.insert(only.span.start);
        }
    }

    /// Writes the values that make two nodes of one kind differ: every field oxc's `ContentEq` compares
    /// that is not a child node, and which optional child is present where two are of one type (spec
    /// SAME-1).
    fn values(&mut self, kind: AstKind<'_>) {
        match kind {
            AstKind::IdentifierName(node) => self.name(node.name.as_str()),
            AstKind::IdentifierReference(node) => self.name(node.name.as_str()),
            AstKind::BindingIdentifier(node) => self.name(node.name.as_str()),
            AstKind::LabelIdentifier(node) => self.name(node.name.as_str()),
            AstKind::PrivateIdentifier(node) => self.name(node.name.as_str()),
            AstKind::JSXIdentifier(node) => self.name(node.name.as_str()),
            AstKind::TSIndexSignatureName(node) => self.name(node.name.as_str()),
            // A JSX attribute's value written over lines keeps its line endings, as JSX has no escapes
            // (spec DUP-5).
            AstKind::StringLiteral(node) if self.attribute => {
                self.string(&fed(&node.value.display().to_string()))
            }
            AstKind::StringLiteral(node) => self.string(&node.value.display().to_string()),
            AstKind::TemplateElement(node) => {
                self.string(&template(node));
                self.flags(format_args!("{}", node.tail));
            }
            AstKind::JSXText(node) => self.string(&rendered(node.value.as_str())),
            AstKind::RegExpLiteral(node) => self.string(&format!(
                "/{}/{:?}",
                node.regex.pattern.text.as_str(),
                node.regex.flags
            )),
            AstKind::Directive(node) => self.string(node.directive.as_str()),
            AstKind::NumericLiteral(node) => self.number(&node.value.to_bits().to_le_bytes()),
            AstKind::BigIntLiteral(node) => self.number(node.value.as_str().as_bytes()),
            AstKind::BooleanLiteral(node) => self.flags(format_args!("{}", node.value)),
            AstKind::AccessorProperty(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.computed,
                    &node.definite,
                    &node.r#override,
                    &node.r#static,
                    &node.r#type
                )
            )),
            AstKind::ArrowFunctionExpression(node) => {
                self.flags(format_args!("{}", node.r#async));
                self.bare(node);
            }
            AstKind::AssignmentExpression(node) => {
                self.flags(format_args!("{:?}", node.operator));
            }
            AstKind::AssignmentTargetPropertyProperty(node) => {
                self.flags(format_args!("{}", node.computed));
            }
            AstKind::BinaryExpression(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::BindingProperty(node) => {
                self.flags(format_args!("{:?}", (&node.computed, &node.shorthand)));
            }
            AstKind::CallExpression(node) => self.flags(format_args!("{}", node.optional)),
            AstKind::Class(node) => self.flags(format_args!(
                "{:?}",
                (&node.declare, &node.r#abstract, &node.r#type)
            )),
            AstKind::ComputedMemberExpression(node) => {
                self.flags(format_args!("{}", node.optional));
            }
            AstKind::ExportAllDeclaration(node) => {
                self.flags(format_args!("{:?}", node.export_kind));
            }
            AstKind::ExportNamedDeclaration(node) => {
                self.flags(format_args!("{:?}", node.export_kind));
            }
            AstKind::ExportSpecifier(node) => self.flags(format_args!("{:?}", node.export_kind)),
            AstKind::ForOfStatement(node) => self.flags(format_args!("{}", node.r#await)),
            AstKind::ForStatement(node) => self.flags(format_args!(
                "{:?}",
                (
                    node.init.is_some(),
                    node.test.is_some(),
                    node.update.is_some()
                )
            )),
            AstKind::FormalParameter(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.optional,
                    &node.r#override,
                    &node.readonly
                )
            )),
            AstKind::FormalParameters(node) => self.flags(format_args!("{:?}", node.kind)),
            AstKind::Function(node) => self.flags(format_args!(
                "{:?}",
                (&node.declare, &node.generator, &node.r#async, &node.r#type)
            )),
            AstKind::ImportDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.import_kind, &node.phase)));
            }
            AstKind::ImportExpression(node) => self.flags(format_args!("{:?}", node.phase)),
            AstKind::ImportSpecifier(node) => self.flags(format_args!("{:?}", node.import_kind)),
            AstKind::JSDocNullableType(node) => self.flags(format_args!("{}", node.postfix)),
            AstKind::JSDocNonNullableType(node) => self.flags(format_args!("{}", node.postfix)),
            AstKind::LogicalExpression(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::MethodDefinition(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.computed,
                    &node.kind,
                    &node.optional,
                    &node.r#override,
                    &node.r#static,
                    &node.r#type
                )
            )),
            AstKind::NewExpression(node) if node.arguments.is_empty() => {
                self.dropped.empty.insert(node.span.end);
            }
            AstKind::ObjectProperty(node) => self.flags(format_args!(
                "{:?}",
                (&node.computed, &node.kind, &node.method, &node.shorthand)
            )),
            AstKind::PrivateFieldExpression(node) => {
                self.flags(format_args!("{}", node.optional));
            }
            AstKind::PropertyDefinition(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.computed,
                    &node.declare,
                    &node.definite,
                    &node.optional,
                    &node.r#override,
                    &node.r#static,
                    &node.r#type,
                    &node.readonly
                )
            )),
            AstKind::StaticMemberExpression(node) => {
                self.flags(format_args!("{}", node.optional));
            }
            AstKind::TSConstructorType(node) => self.flags(format_args!("{}", node.r#abstract)),
            AstKind::TSEnumDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.declare, &node.r#const)));
            }
            AstKind::TSExternalModuleDeclaration(node) => {
                self.flags(format_args!("{}", node.declare));
            }
            AstKind::TSGlobalDeclaration(node) => self.flags(format_args!("{}", node.declare)),
            AstKind::TSImportEqualsDeclaration(node) => {
                self.flags(format_args!("{:?}", node.import_kind));
            }
            AstKind::TSIndexSignature(node) => {
                self.flags(format_args!("{:?}", (&node.r#static, &node.readonly)));
            }
            AstKind::TSInterfaceDeclaration(node) => self.flags(format_args!("{}", node.declare)),
            AstKind::TSMappedType(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.optional,
                    &node.readonly,
                    node.name_type.is_some(),
                    node.type_annotation.is_some()
                )
            )),
            AstKind::TSMethodSignature(node) => self.flags(format_args!(
                "{:?}",
                (&node.computed, &node.kind, &node.optional)
            )),
            AstKind::TSNamedTupleMember(node) => self.flags(format_args!("{}", node.optional)),
            AstKind::TSNamespaceDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.declare, &node.kind)));
            }
            AstKind::TSPropertySignature(node) => self.flags(format_args!(
                "{:?}",
                (&node.computed, &node.optional, &node.readonly)
            )),
            AstKind::TSTypeAliasDeclaration(node) => self.flags(format_args!("{}", node.declare)),
            AstKind::TSTypeOperator(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::TSTypeParameter(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.r#in,
                    &node.out,
                    &node.r#const,
                    node.constraint.is_some(),
                    node.default.is_some()
                )
            )),
            AstKind::TSTypePredicate(node) => self.flags(format_args!("{}", node.asserts)),
            AstKind::UnaryExpression(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::UpdateExpression(node) => {
                self.flags(format_args!("{:?}", (&node.operator, &node.prefix)));
            }
            AstKind::VariableDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.declare, &node.kind)));
            }
            AstKind::VariableDeclarator(node) => self.flags(format_args!("{}", node.definite)),
            AstKind::WithClause(node) => self.flags(format_args!("{:?}", node.keyword)),
            AstKind::YieldExpression(node) => self.flags(format_args!("{}", node.delegate)),
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Former<'_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            // Parentheses that change nothing are not part of the shape (spec SAME-1).
            AstKind::ParenthesizedExpression(node) => self.unwrapped(node.span),
            AstKind::TSParenthesizedType(node) => self.unwrapped(node.span),
            // A JSX text that holds only white space is not part of the shape (spec SAME-1).
            AstKind::JSXText(node) if rendered(node.value.as_str()).is_empty() => {}
            _ => {
                self.put(b'w', format!("{:?}", kind.ty()).as_bytes());
                self.values(kind);
            }
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::ParenthesizedExpression(_) | AstKind::TSParenthesizedType(_) => {}
            AstKind::JSXText(node) if rendered(node.value.as_str()).is_empty() => {}
            _ => self.stream.push(CLOSE),
        }
    }

    fn visit_statements(&mut self, statements: &ArenaVec<'a, Statement<'a>>) {
        let list = self.lists.len();

        self.lists.push(Vec::new());

        for statement in statements {
            // An import never joins a run (spec COPY-2).
            let joins = !matches!(
                statement,
                Statement::ImportDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
                    | Statement::ExportFromDeclaration(_)
                    | Statement::TSImportEqualsDeclaration(_)
            );

            self.item(list, statement.span(), joins, |former| {
                former.visit_statement(statement);
            });
        }
    }

    fn visit_class_elements(&mut self, elements: &ArenaVec<'a, ClassElement<'a>>) {
        let list = self.lists.len();

        self.lists.push(Vec::new());

        for element in elements {
            self.item(list, element.span(), true, |former| {
                former.visit_class_element(element);
            });
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let test = self.chained
            || functions::callee(&call.callee)
                .is_some_and(|chain| self.lens.test_callees.contains(&chain));

        if !test {
            walk::walk_call_expression(self, call);

            return;
        }

        // A statement that holds a test callee's call joins no run (spec COPY-3).
        if !self.chained {
            for &(list, index) in &self.open {
                self.lists[list][index].joins = false;
            }
        }

        let kind = AstKind::CallExpression(self.alloc(call));
        let chained = self.chained;

        self.enter_node(kind);
        self.chained = true;
        self.visit_expression(&call.callee);
        self.chained = false;

        if let Some(arguments) = &call.type_arguments {
            self.visit_ts_type_parameter_instantiation(arguments);
        }

        // Nothing inside a test callee's callback is looked at (spec COPY-3).
        for argument in call.arguments.iter().filter(|argument| {
            !matches!(
                argument,
                Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_)
            )
        }) {
            self.visit_argument(argument);
        }

        self.chained = chained;
        self.leave_node(kind);
    }

    fn visit_jsx_attribute_value(&mut self, value: &JSXAttributeValue<'a>) {
        self.attribute = matches!(value, JSXAttributeValue::StringLiteral(_));
        walk::walk_jsx_attribute_value(self, value);
        self.attribute = false;
    }

    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
        // A key written in quotes is the name it spells (spec SAME-1).
        if let PropertyKey::StringLiteral(literal) = key {
            self.put(b'w', b"IdentifierName");
            self.name(&literal.value.display().to_string());
            self.stream.push(CLOSE);
        } else {
            walk::walk_property_key(self, key);
        }
    }
}

/// The tokens a size counts, with the lines they start on (spec SIZE-1).
struct Counted {
    /// Where each starts, in order.
    starts: Vec<u32>,
    /// The line each starts on.
    on: Vec<usize>,
    /// How many of the tokens before each, and before the end, start on a line the token before them
    /// does not.
    breaks: Vec<u64>,
}

impl Counted {
    /// The tokens of a program the fixed form counts: all but every `;`, a `,` just before a closing
    /// bracket, the parentheses the walk dropped, an empty `()` after `new`, the parentheses around an
    /// arrow function's lone parameter, and a JSX text of white space only.
    fn of(tokens: &[Token], text: &str, lines: &Lines<'_>, dropped: &Dropped) -> Self {
        let tokens: Vec<&Token> = tokens
            .iter()
            .filter(|token| !matches!(token.kind(), Kind::Skip | Kind::Eof))
            .collect();
        let mut kept = vec![true; tokens.len()];

        for (index, token) in tokens.iter().enumerate() {
            let before = index
                .checked_sub(1)
                .and_then(|before| tokens.get(before))
                .map(|before| before.kind());
            let next = tokens.get(index + 1).map(|next| next.kind());

            match token.kind() {
                Kind::Semicolon => kept[index] = false,
                Kind::Comma if matches!(next, Some(Kind::RParen | Kind::RBrack | Kind::RCurly)) => {
                    kept[index] = false;
                }
                Kind::LParen if dropped.opens.contains(&token.start()) => kept[index] = false,
                Kind::RParen if dropped.closes.contains(&token.end()) => kept[index] = false,
                Kind::RParen
                    if dropped.empty.contains(&token.end())
                        && matches!(before, Some(Kind::LParen)) =>
                {
                    kept[index] = false;
                    kept[index - 1] = false;
                }
                Kind::JSXText => {
                    let offset = |at: u32| usize::try_from(at).unwrap_or(usize::MAX);
                    let piece = text.get(offset(token.start())..offset(token.end()));

                    if piece.is_none_or(|piece| piece.trim().is_empty()) {
                        kept[index] = false;
                    }
                }
                _ => {}
            }

            if dropped.lone.contains(&token.start())
                && matches!(before, Some(Kind::LParen))
                && matches!(next, Some(Kind::RParen))
            {
                kept[index - 1] = false;
                kept[index + 1] = false;
            }
        }

        let mut starts = Vec::new();
        let mut on = Vec::new();

        for (token, kept) in tokens.iter().zip(kept) {
            if kept {
                starts.push(token.start());
                on.push(lines.at(token.start()));
            }
        }

        let mut breaks = Vec::with_capacity(on.len() + 1);

        breaks.push(0);

        for (index, line) in on.iter().enumerate() {
            let new = index > 0 && on[index - 1] != *line;

            breaks.push(breaks[index] + u64::from(new));
        }

        Self { starts, on, breaks }
    }

    /// One statement, with its digest, its size and its place.
    fn item(&self, raw: &Raw, stream: &[u8], lines: &Lines<'_>) -> Item {
        let first = self.starts.partition_point(|start| *start < raw.span.start);
        let after = self.starts.partition_point(|start| *start < raw.span.end);
        let digest = Sha256::digest(&stream[raw.from..raw.to]);
        let mut kept = [0; DIGEST];

        kept.copy_from_slice(&digest[..DIGEST]);

        Item {
            digest: kept,
            joins: raw.joins,
            tokens: u64::try_from(after - first).unwrap_or(u64::MAX),
            lines: (after > first).then(|| Held {
                first: self.on[first],
                last: self.on[after - 1],
                count: 1 + self.breaks[after] - self.breaks[first + 1],
            }),
            start: raw.span.start,
            end: raw.span.end,
            line: lines.at(raw.span.start),
            end_line: lines.at(raw.span.end.saturating_sub(1)),
        }
    }
}
