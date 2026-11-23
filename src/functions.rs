//! What a function is, its name, and its scores: every function of a script, found in the parse that
//! finds its imports, with the classes and object literals around it (complexity spec FN-1 to FN-5,
//! SCORE-1 to SCORE-3).
//!
//! The walk records units in source order: each function, class and object literal, with the unit
//! around it. A file's name paths are worked out once all its units are known, since a class or an
//! object literal takes part in them only where a function is inside it (spec FN-5).

use std::collections::BTreeMap;

use oxc_ast::AstKind;
use oxc_ast::ast::{
    self, Argument, ArrowFunctionExpression, AssignmentExpression, AssignmentTarget,
    BindingIdentifier, BindingPattern, BreakStatement, CallExpression, CatchClause, ChainElement,
    ChainExpression, Class, ConditionalExpression, ContinueStatement, DoWhileStatement, Expression,
    ForInStatement, ForOfStatement, ForStatement, FormalParameters, IfStatement, JSXElement,
    JSXFragment, LogicalExpression, LogicalOperator, MethodDefinitionKind, ObjectExpression,
    Program, PropertyKey, PropertyKind, Statement, SwitchCase, SwitchStatement, WhileStatement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{NodeId, ScopeFlags, Semantic};
use oxc_span::{GetSpan, Span};

use crate::imports::{DEFAULT, Lines};

/// How many properties a component's first parameter may take apart before each more adds to its
/// cognitive complexity (fallow's `PROP_COUNT_FLOOR`; spec SCORE-2).
const PROPS_FLOOR: usize = 4;

/// One function, class or object literal of a file, as the walk found it.
#[derive(Debug, PartialEq, Eq)]
pub struct Unit {
    /// The unit around it, by its place in the file's list; none at the file's top.
    pub parent: Option<usize>,
    /// Its own name, where it has one (spec FN-4).
    pub name: Option<String>,
    /// Where a function is and its scores; none for a class or an object literal.
    pub scored: Option<Scored>,
    /// Whether it is a function, or a function is somewhere inside it (spec FN-5).
    pub holds: bool,
}

/// Where a function is, and its scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scored {
    /// The line it is placed on, counted from one (spec CPX-2).
    pub line: usize,
    /// The column it is placed at, counted from one in UTF-16 code units.
    pub column: usize,
    /// Its cyclomatic complexity (spec SCORE-1).
    pub cyclomatic: u64,
    /// Its cognitive complexity (spec SCORE-1).
    pub cognitive: u64,
}

/// One part of a name path (spec FN-5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    /// Its name; none for an unnamed part.
    pub name: Option<String>,
    /// Its number among the parts of the same name, or among the unnamed ones, in what is around it.
    pub number: usize,
}

/// One function, with its name path (spec FN-5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// The names of what is around it, from the outside in, then its own.
    pub path: Vec<Part>,
    /// Where it is, and its scores.
    pub scored: Scored,
}

impl Function {
    /// Its name path as a person reads it, as in `Parser > read #2 > <anonymous 1>` (spec FN-5).
    pub fn readable(&self) -> String {
        self.path
            .iter()
            .map(|part| match (&part.name, part.number) {
                (Some(name), 1) => name.clone(),
                (Some(name), number) => format!("{name} #{number}"),
                (None, number) => format!("<anonymous {number}>"),
            })
            .collect::<Vec<_>>()
            .join(" > ")
    }
}

/// The functions of a file's units, in source order, each with its name path. Within what is around
/// them, the parts that take part are counted in source order: a name's uses 1, 2 and on, and the
/// unnamed ones among themselves (spec FN-5).
pub fn named(units: &[Unit]) -> Vec<Function> {
    let mut paths: Vec<Option<Vec<Part>>> = Vec::with_capacity(units.len());
    let mut counts: BTreeMap<(Option<usize>, Option<&str>), usize> = BTreeMap::new();
    let mut functions = Vec::new();

    for unit in units {
        let around = match unit.parent {
            Some(parent) => paths.get(parent).cloned().flatten(),
            None => Some(Vec::new()),
        };
        // A class or an object literal with no function inside takes no place (spec FN-5).
        let path = around.filter(|_| unit.holds).map(|mut path| {
            let count = counts
                .entry((unit.parent, unit.name.as_deref()))
                .or_default();

            *count += 1;
            path.push(Part {
                name: unit.name.clone(),
                number: *count,
            });
            path
        });

        if let (Some(path), Some(scored)) = (&path, unit.scored) {
            functions.push(Function {
                path: path.clone(),
                scored,
            });
        }

        paths.push(path);
    }

    functions
}

/// Adds the units of a Vue component's script block to the component's, so that its blocks count as
/// one file, with their places counted in the file: `above` lines above the block's content, and
/// `indent` columns before it on its first line (spec FN-5).
pub fn append(units: &mut Vec<Unit>, block: Vec<Unit>, above: usize, indent: usize) {
    let offset = units.len();

    units.extend(block.into_iter().map(|unit| Unit {
        parent: unit.parent.map(|parent| parent + offset),
        scored: unit.scored.map(|scored| Scored {
            line: scored.line + above,
            column: if scored.line == 1 {
                scored.column + indent
            } else {
                scored.column
            },
            ..scored
        }),
        ..unit
    }));
}

/// The units of a parsed program, with what the binding analysis knows of its nodes (spec FN-3).
pub fn units<'a>(program: &Program<'a>, semantic: &Semantic<'a>, lines: &Lines<'_>) -> Vec<Unit> {
    let mut scorer = Scorer {
        semantic,
        lines,
        units: Vec::new(),
        open: Vec::new(),
        frames: Vec::new(),
    };

    scorer.visit_program(program);
    scorer.units
}

/// The scores so far of a function the walk is inside.
struct Frame {
    /// The function, by its place among the units.
    unit: usize,
    /// Its cyclomatic complexity so far.
    cyclomatic: u64,
    /// Its cognitive complexity so far.
    cognitive: u64,
    /// How many of the structures that nest are around the walk's place in it.
    nesting: u64,
    /// The operator of the last `&&`, `||` or `??` it counted, until a run of them ends (spec
    /// SCORE-2).
    last: Option<LogicalOperator>,
    /// Whether it renders JSX.
    jsx: bool,
    /// Whether its name is shaped like a React component's or hook's.
    react: bool,
    /// How many properties its first parameter takes apart, where it is an object pattern.
    props: usize,
}

impl Frame {
    /// What a component that takes many properties apart adds to its cognitive complexity: each
    /// property past the floor, where it renders JSX (fallow's `fold_component_prop_count`; spec
    /// SCORE-2).
    fn wide(&self) -> u64 {
        if self.jsx {
            u64::try_from(self.props.saturating_sub(PROPS_FLOOR)).unwrap_or(u64::MAX)
        } else {
            0
        }
    }
}

/// Walks a program and records its units, with each function's scores.
struct Scorer<'s, 'a> {
    semantic: &'s Semantic<'a>,
    lines: &'s Lines<'s>,
    units: Vec<Unit>,
    /// The units the walk is inside, innermost last.
    open: Vec<usize>,
    /// The functions the walk is inside, innermost last.
    frames: Vec<Frame>,
}

impl<'a> Scorer<'_, 'a> {
    /// Records a unit the walk enters: a function, with its parameters, which takes a frame of scores
    /// of its own, or a class or an object literal. `own` is its own identifier, where it has one.
    fn enter(
        &mut self,
        node: NodeId,
        span: Span,
        own: Option<&BindingIdentifier<'a>>,
        params: Option<&FormalParameters<'a>>,
    ) {
        let (name, at) = match own {
            Some(identifier) => (Some(identifier.name.to_string()), identifier.span.start),
            None => self.name(node, span),
        };
        let place = self.units.len();
        let scored = params.map(|_| {
            let (line, column) = self.lines.place(at);

            Scored {
                line,
                column,
                cyclomatic: 1,
                cognitive: 0,
            }
        });

        if params.is_some() {
            // Each unit around a function holds one (spec FN-5).
            for open in self.open.iter().rev() {
                if self.units[*open].holds {
                    break;
                }

                self.units[*open].holds = true;
            }

            let react = self.react(node, span, own);
            let props = params
                .and_then(|params| params.items.first())
                .map_or(0, |first| match &first.pattern {
                    BindingPattern::ObjectPattern(object) => object.properties.len(),
                    _ => 0,
                });

            self.frames.push(Frame {
                unit: place,
                cyclomatic: 1,
                cognitive: 0,
                nesting: 0,
                last: None,
                jsx: false,
                react,
                props,
            });
        }

        self.units.push(Unit {
            parent: self.open.last().copied(),
            name,
            scored,
            holds: params.is_some(),
        });
        self.open.push(place);
    }

    /// Records that the walk leaves the innermost unit, with a function's scores.
    fn leave(&mut self, function: bool) {
        if function
            && let Some(frame) = self.frames.pop()
            && let Some(scored) = &mut self.units[frame.unit].scored
        {
            scored.cyclomatic = frame.cyclomatic;
            scored.cognitive = frame.cognitive + frame.wide();
        }

        self.open.pop();
    }

    /// What holds a unit, past the parentheses and type assertions around it, with the span of the
    /// outermost of those, which is what the holder holds.
    fn holder(&self, node: NodeId, span: Span) -> (Span, AstKind<'a>) {
        let nodes = self.semantic.nodes();
        let mut node = node;
        let mut span = span;

        loop {
            let parent = nodes.parent_id(node);
            let kind = nodes.kind(parent);

            match kind {
                AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSTypeAssertion(_) => {
                    node = parent;
                    span = kind.span();
                }
                _ => return (span, kind),
            }
        }
    }

    /// A unit's name, from what holds it, and the offset it is placed at: the key, where the name is
    /// a method's or a property's, and otherwise the unit's own start (spec FN-4, CPX-2).
    fn name(&self, node: NodeId, span: Span) -> (Option<String>, u32) {
        let (outer, holder) = self.holder(node, span);

        if let Some((name, at)) = declared(holder, outer) {
            return (Some(name), at.unwrap_or(span.start));
        }

        let name = match holder {
            AstKind::AssignmentExpression(assignment) if assignment.right.span() == outer => {
                target(&assignment.left)
            }
            AstKind::CallExpression(call) if passed(&call.arguments, outer) => {
                called(&call.callee, &call.arguments)
            }
            AstKind::NewExpression(call) if passed(&call.arguments, outer) => {
                called(&call.callee, &call.arguments).map(|called| format!("new {called}"))
            }
            _ => None,
        };

        (name, span.start)
    }

    /// Whether a function's name is shaped like a React component's or hook's, read as fallow reads
    /// it: its own identifier, or else what declares it, or the call it is passed to, by name; never a
    /// name taken from a call (fallow's `frame_name_is_react_shaped`; spec SCORE-2).
    fn react(&self, node: NodeId, span: Span, own: Option<&BindingIdentifier<'a>>) -> bool {
        let mut node = node;
        let mut span = span;
        let name = own
            .map(|identifier| identifier.name.to_string())
            .or_else(|| {
                loop {
                    let (outer, holder) = self.holder(node, span);

                    if let Some((name, _)) = declared(holder, outer) {
                        break Some(name);
                    }

                    match holder {
                        AstKind::CallExpression(call) if passed(&call.arguments, outer) => {
                            node = call.node_id.get();
                            span = call.span;
                        }
                        _ => break None,
                    }
                }
            });

        name.is_some_and(|name| {
            name.starts_with(|first: char| first.is_ascii_uppercase()) || hook(&name)
        })
    }

    /// The innermost function's frame, where the walk is inside one.
    fn frame(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    /// Adds a path to the innermost function's cyclomatic complexity (spec SCORE-1, SCORE-2).
    fn branch(&mut self) {
        if let Some(frame) = self.frame() {
            frame.cyclomatic += 1;
        }
    }

    /// Adds one to the innermost function's cognitive complexity, whatever its nesting.
    fn flat(&mut self) {
        if let Some(frame) = self.frame() {
            frame.cognitive += 1;
        }
    }

    /// Adds one and the nesting to the innermost function's cognitive complexity.
    fn structural(&mut self) {
        if let Some(frame) = self.frame() {
            frame.cognitive += 1 + frame.nesting;
        }
    }

    /// Walks what the closure walks one level of nesting deeper in the innermost function.
    fn nested(&mut self, walk: impl FnOnce(&mut Self)) {
        if let Some(frame) = self.frame() {
            frame.nesting += 1;
        }

        walk(self);

        if let Some(frame) = self.frame() {
            frame.nesting -= 1;
        }
    }

    /// A loop: a path, and one and the nesting (spec SCORE-1).
    fn looped(&mut self) {
        self.branch();
        self.structural();
    }

    /// An `if` with the `else if`s and the `else` after it. Each `if` adds a path. The first adds one
    /// and its nesting to the cognitive score, and each `else if` and the `else` one, and each body is
    /// one level deeper than the first `if`; each test is at its level (fallow's `visit_if_chain`;
    /// spec SCORE-1, SCORE-2).
    fn if_chain(&mut self, statement: &IfStatement<'a>, alternate: bool) {
        self.branch();

        if alternate {
            self.flat();
        } else {
            self.structural();
        }

        self.visit_expression(&statement.test);
        self.nested(|scorer| scorer.visit_statement(&statement.consequent));

        match &statement.alternate {
            Some(Statement::IfStatement(next)) => self.if_chain(next, true),
            Some(otherwise) => {
                self.flat();
                self.nested(|scorer| scorer.visit_statement(otherwise));
            }
            None => {}
        }
    }
}

/// Whether a name is a React hook's: `use` and then a capital letter (fallow's `is_hook_callee`).
fn hook(name: &str) -> bool {
    name.strip_prefix("use")
        .is_some_and(|rest| rest.starts_with(|next: char| next.is_ascii_uppercase()))
}

/// The name a declaration gives what it holds, with the offset the name is placed at where it is a
/// key: the variable it is declared to, the key of an object's property or a class's member, with
/// `get ` or `set ` before an accessor's, or `default` for an anonymous default export. A computed key
/// gives no name (spec FN-4, CPX-2).
fn declared(holder: AstKind<'_>, outer: Span) -> Option<(String, Option<u32>)> {
    let keyed = |key: &PropertyKey<'_>, computed: bool, before: &str| {
        key_name(key, computed).map(|name| (format!("{before}{name}"), Some(key.span().start)))
    };

    match holder {
        AstKind::VariableDeclarator(declarator) if holds(declarator.init.as_ref(), outer) => {
            match &declarator.id {
                BindingPattern::BindingIdentifier(identifier) => {
                    Some((identifier.name.to_string(), None))
                }
                _ => None,
            }
        }
        AstKind::ObjectProperty(property) if property.value.span() == outer => {
            let before = match property.kind {
                PropertyKind::Get => "get ",
                PropertyKind::Set => "set ",
                PropertyKind::Init => "",
            };

            keyed(&property.key, property.computed, before)
        }
        AstKind::MethodDefinition(method) => {
            let before = match method.kind {
                MethodDefinitionKind::Get => "get ",
                MethodDefinitionKind::Set => "set ",
                MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => "",
            };

            keyed(&method.key, method.computed, before)
        }
        AstKind::PropertyDefinition(property) if holds(property.value.as_ref(), outer) => {
            keyed(&property.key, property.computed, "")
        }
        AstKind::AccessorProperty(property) if holds(property.value.as_ref(), outer) => {
            keyed(&property.key, property.computed, "")
        }
        AstKind::ExportDefaultDeclaration(_) => Some((DEFAULT.to_owned(), None)),
        _ => None,
    }
}

/// Whether a value a declaration may hold is the unit of the span given.
fn holds(value: Option<&Expression<'_>>, outer: Span) -> bool {
    value.is_some_and(|value| value.span() == outer)
}

/// Whether the unit of the span given is one of a call's arguments.
fn passed(arguments: &[Argument<'_>], outer: Span) -> bool {
    arguments.iter().any(|argument| argument.span() == outer)
}

/// The name of a key that is not computed: an identifier, a private name with its `#`, or a literal's
/// value.
fn key_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    match key {
        _ if computed => None,
        PropertyKey::PrivateIdentifier(private) => Some(format!("#{}", private.name)),
        _ => key.static_name().map(|name| name.display().to_string()),
    }
}

/// The whole left side of an assignment, as a chain of names (spec FN-4).
fn target(left: &AssignmentTarget<'_>) -> Option<String> {
    match left {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some(identifier.name.to_string())
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            after(&member.object, &format!(".{}", member.property.name))
        }
        AssignmentTarget::ComputedMemberExpression(member) => after(&member.object, "[]"),
        AssignmentTarget::PrivateFieldExpression(member) => {
            after(&member.object, &format!(".#{}", member.field.name))
        }
        AssignmentTarget::TSAsExpression(wrapped) => chain(&wrapped.expression),
        AssignmentTarget::TSSatisfiesExpression(wrapped) => chain(&wrapped.expression),
        AssignmentTarget::TSNonNullExpression(wrapped) => chain(&wrapped.expression),
        AssignmentTarget::TSTypeAssertion(wrapped) => chain(&wrapped.expression),
        _ => None,
    }
}

/// The name a function passed to a call takes from it: the callee with `()`, and inside them the
/// call's first argument where it is a string, written in double quotes (spec FN-4).
fn called(callee: &Expression<'_>, arguments: &[Argument<'_>]) -> Option<String> {
    let first = arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(literal)
        .map(|text| format!("\"{text}\""))
        .unwrap_or_default();

    chain(callee).map(|callee| format!("{callee}({first})"))
}

/// The value of a string literal, or of a template literal with no substitutions.
fn literal(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.display().to_string()),
        Expression::TemplateLiteral(template) => template
            .single_quasi()
            .map(|text| text.display().to_string()),
        _ => None,
    }
}

/// An expression as a chain of names joined by `.`, with `[]` for a computed member, `()` for a call
/// without its arguments, and `new` kept, whatever spacing, comments, parentheses or optional chaining
/// it is written with; none for any other expression (spec FN-4).
fn chain(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::ThisExpression(_) => Some("this".to_owned()),
        Expression::Super(_) => Some("super".to_owned()),
        Expression::StaticMemberExpression(member) => {
            after(&member.object, &format!(".{}", member.property.name))
        }
        Expression::ComputedMemberExpression(member) => after(&member.object, "[]"),
        Expression::PrivateFieldExpression(member) => {
            after(&member.object, &format!(".#{}", member.field.name))
        }
        Expression::CallExpression(call) => after(&call.callee, "()"),
        Expression::NewExpression(call) => {
            chain(&call.callee).map(|callee| format!("new {callee}()"))
        }
        Expression::ChainExpression(chained) => match &chained.expression {
            ChainElement::CallExpression(call) => after(&call.callee, "()"),
            ChainElement::TSNonNullExpression(wrapped) => chain(&wrapped.expression),
            ChainElement::StaticMemberExpression(member) => {
                after(&member.object, &format!(".{}", member.property.name))
            }
            ChainElement::ComputedMemberExpression(member) => after(&member.object, "[]"),
            ChainElement::PrivateFieldExpression(member) => {
                after(&member.object, &format!(".#{}", member.field.name))
            }
        },
        Expression::ParenthesizedExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSAsExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSSatisfiesExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSNonNullExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSTypeAssertion(wrapped) => chain(&wrapped.expression),
        Expression::TSInstantiationExpression(wrapped) => chain(&wrapped.expression),
        _ => None,
    }
}

/// The chain of an expression with the text given after it.
fn after(object: &Expression<'_>, text: &str) -> Option<String> {
    chain(object).map(|object| format!("{object}{text}"))
}

impl<'a> Visit<'a> for Scorer<'_, 'a> {
    fn visit_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        // A declaration with no body is not a function: an overload, a `declare function` or an
        // abstract method (spec FN-1).
        let params = function.body.as_ref().map(|_| &*function.params);

        if params.is_some() {
            self.enter(
                function.node_id.get(),
                function.span,
                function.id.as_ref(),
                params,
            );
        }

        walk::walk_function(self, function, flags);

        if params.is_some() {
            self.leave(true);
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.enter(arrow.node_id.get(), arrow.span, None, Some(&*arrow.params));
        walk::walk_arrow_function_expression(self, arrow);
        self.leave(true);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.enter(class.node_id.get(), class.span, class.id.as_ref(), None);
        walk::walk_class(self, class);
        self.leave(false);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        self.enter(object.node_id.get(), object.span, None, None);
        walk::walk_object_expression(self, object);
        self.leave(false);
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
        self.if_chain(statement, false);
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.looped();

        if let Some(init) = &statement.init {
            self.visit_for_statement_init(init);
        }

        if let Some(test) = &statement.test {
            self.visit_expression(test);
        }

        if let Some(update) = &statement.update {
            self.visit_expression(update);
        }

        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        self.looped();
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.looped();
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_while_statement(&mut self, statement: &WhileStatement<'a>) {
        self.looped();
        self.visit_expression(&statement.test);
        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_do_while_statement(&mut self, statement: &DoWhileStatement<'a>) {
        self.looped();
        self.nested(|scorer| scorer.visit_statement(&statement.body));
        self.visit_expression(&statement.test);
    }

    // A `switch` adds one and its nesting once, and each `case` with a test a path, an empty one
    // included (fallow's `visit_switch_statement` and `visit_switch_case`).
    fn visit_switch_statement(&mut self, statement: &SwitchStatement<'a>) {
        self.structural();
        self.visit_expression(&statement.discriminant);
        self.nested(|scorer| scorer.visit_switch_cases(&statement.cases));
    }

    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        if case.test.is_some() {
            self.branch();
        }

        walk::walk_switch_case(self, case);
    }

    // A `try` and a `finally` add nothing and nest nothing; a `catch` adds a path, and one and its
    // nesting.
    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.branch();
        self.structural();
        self.nested(|scorer| walk::walk_catch_clause(scorer, clause));
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        self.branch();
        self.structural();
        self.visit_expression(&conditional.test);
        self.nested(|scorer| {
            scorer.visit_expression(&conditional.consequent);
            scorer.visit_expression(&conditional.alternate);
        });
    }

    // Each `&&`, `||` and `??` adds a path. One adds to the cognitive score when its operator is not
    // the last one counted, and the last one is forgotten after an operator neither of whose sides is
    // another, parentheses not seen through (fallow's `visit_logical_expression`; spec SCORE-2).
    fn visit_logical_expression(&mut self, logical: &LogicalExpression<'a>) {
        self.branch();

        if let Some(frame) = self.frame()
            && frame.last != Some(logical.operator)
        {
            frame.cognitive += 1;
            frame.last = Some(logical.operator);
        }

        walk::walk_logical_expression(self, logical);

        let is_logical = |side: &Expression<'_>| matches!(side, Expression::LogicalExpression(_));

        if !is_logical(&logical.left)
            && !is_logical(&logical.right)
            && let Some(frame) = self.frame()
        {
            frame.last = None;
        }
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if assignment.operator.is_logical() {
            self.branch();
        }

        walk::walk_assignment_expression(self, assignment);
    }

    // A chain adds a path where its outermost link is optional, and its other links add nothing
    // (fallow's `visit_chain_expression`; spec SCORE-2).
    fn visit_chain_expression(&mut self, chained: &ChainExpression<'a>) {
        let optional = match &chained.expression {
            ChainElement::CallExpression(call) => call.optional,
            ChainElement::TSNonNullExpression(_) => false,
            ChainElement::ComputedMemberExpression(member) => member.optional,
            ChainElement::StaticMemberExpression(member) => member.optional,
            ChainElement::PrivateFieldExpression(member) => member.optional,
        };

        if optional {
            self.branch();
        }

        walk::walk_chain_expression(self, chained);
    }

    fn visit_break_statement(&mut self, statement: &BreakStatement<'a>) {
        if statement.label.is_some() {
            self.flat();
        }

        walk::walk_break_statement(self, statement);
    }

    fn visit_continue_statement(&mut self, statement: &ContinueStatement<'a>) {
        if statement.label.is_some() {
            self.flat();
        }

        walk::walk_continue_statement(self, statement);
    }

    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if let Some(frame) = self.frame() {
            frame.jsx = true;
        }

        walk::walk_jsx_element(self, element);
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        if let Some(frame) = self.frame() {
            frame.jsx = true;
        }

        walk::walk_jsx_fragment(self, fragment);
    }

    // A hook's call adds one to a component's or a hook's cognitive score (fallow's `record_hook`;
    // spec SCORE-2).
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && hook(callee.name.as_str())
            && let Some(frame) = self.frame()
            && frame.react
        {
            frame.cognitive += 1;
        }

        walk::walk_call_expression(self, call);
    }
}
