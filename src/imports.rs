//! What an import is, and what a script's names are: parsing a script and finding every import in it,
//! the names each brings in, what the script exports, and what of each import's target it uses (spec
//! PARSE-1 to PARSE-4, IMP-1 to IMP-4, USE-1, USE-2).

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, CallExpression, Declaration, ExportAllDeclaration,
    ExportDefaultDeclarationKind, ExportFromDeclaration, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, ImportExpression, JSXMemberExpressionObject, ModuleExportName,
    Program, Statement, StringLiteral, TSImportEqualsDeclaration, TSImportType,
    TSImportTypeQualifier, TSModuleReference, TSTypeName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{NodeId, Semantic, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};

/// The name an import or an export has for the whole of a file: its namespace, or all it exports.
pub const EVERY: &str = "*";

/// The name of a file's default export.
pub const DEFAULT: &str = "default";

/// How an import is written (spec IMP-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// `import … from '…'`, or `import '…'`.
    Import,
    /// `export … from '…'`.
    Export,
    /// `export * from '…'`, and `export * as name from '…'`.
    ExportAll,
    /// `import name = require('…')`.
    ImportEquals,
    /// `import('…')` with a literal.
    Dynamic,
    /// `require('…')` with a literal, where `require` is the bare name.
    Require,
    /// `import('…')` written where a type is expected.
    TypeImport,
    /// The `src` of a Vue component's script block, or of an HTML page's script element.
    Src,
}

impl Form {
    /// The form's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Export => "export",
            Self::ExportAll => "export-all",
            Self::ImportEquals => "import-equals",
            Self::Dynamic => "dynamic",
            Self::Require => "require",
            Self::TypeImport => "type-import",
            Self::Src => "src",
        }
    }
}

/// One name an import brings in (spec USE-1).
#[derive(Debug, PartialEq, Eq)]
pub struct Name {
    /// The export of the target it names: the export's name, `default`, or `*` for the namespace.
    pub name: String,
    /// The binding it makes in the importing file, where it makes one.
    pub local: Option<String>,
    /// The names the importing file offers it under again, each of which passes a use on to the target
    /// (spec USE-2, USE-2a).
    pub exported: Vec<String>,
}

/// One use an import makes of its target (spec USE-2).
#[derive(Debug, PartialEq, Eq)]
pub struct Used {
    /// The export used, or none for every export of the target.
    pub name: Option<String>,
    /// The line the use counts from, counted from one.
    pub line: usize,
}

/// One import.
#[derive(Debug, PartialEq, Eq)]
pub struct Import {
    /// The specifier as it is written.
    pub specifier: String,
    /// How it is written.
    pub form: Form,
    /// Whether it brings in types alone: `import type`, `export type`, a type import, or an `import`
    /// or `export … from` all of whose names are marked `type`.
    pub type_only: bool,
    /// The line it begins on, counted from one.
    pub line: usize,
    /// The names it brings in, in the order they are written (spec USE-1).
    pub names: Vec<Name>,
    /// What of its target the importing file uses through it (spec USE-2).
    pub uses: Vec<Used>,
}

impl Import {
    /// Whether the specifier carries a query or a fragment: a `?`, or a `#` that is not its first
    /// character (spec IMP-4).
    pub fn has_query(&self) -> bool {
        self.specifier.contains('?')
            || self
                .specifier
                .get(1..)
                .is_some_and(|rest| rest.contains('#'))
    }
}

/// One name a file's own `export` statements offer, declared in the file (spec USE-1).
#[derive(Debug, PartialEq, Eq)]
pub struct Export {
    /// Its name; `default` for the default export.
    pub name: String,
    /// Whether it is a type, an interface or a type alias or a namespace of types alone, and not a
    /// value.
    pub is_type: bool,
    /// The line its name is on, counted from one.
    pub line: usize,
    /// The column its name begins at, counted from one in UTF-16 code units.
    pub column: usize,
    /// Whether the file itself refers to it, apart from exporting it (spec USE-2).
    pub used_in_file: bool,
}

/// What reading a script found.
#[derive(Debug, Default)]
pub struct Read {
    /// Its imports, in the order they are written.
    pub imports: Vec<Import>,
    /// How many calls of `import(…)` or `require(…)` have an argument that cannot be followed
    /// (spec IMP-2).
    pub computed: usize,
    /// The names its own `export` statements offer, in the order they are written (spec USE-1).
    pub exports: Vec<Export>,
    /// The uses a file makes of its own exports from outside its scripts, as a Vue component's
    /// template does (spec USE-4).
    pub named: Vec<Used>,
}

/// Why a script did not parse: the line of the first syntax error, where there is one, and its message.
#[derive(Debug)]
pub struct Broken {
    /// The line, counted from one.
    pub line: Option<usize>,
    /// What the parser said.
    pub message: String,
}

/// Parses a script, as a module first and then as a script, and finds its imports and its names (spec
/// PARSE-1).
pub fn read(path: &str, text: &str) -> Result<Read, Broken> {
    read_as(text, source_type(path), false)
}

/// Parses a Vue component's script block in the language its `lang` names: `ts`, `tsx`, `js` or
/// `jsx`, and JavaScript when it names none; nothing for a block in any other language (spec PARSE-3).
pub fn read_block(text: &str, lang: Option<&str>) -> Option<Result<Read, Broken>> {
    let source_type = match lang {
        None | Some("js" | "jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some(_) => return None,
    };

    Some(read_as(text, source_type, true))
}

/// Parses statements of an MDX document as a JavaScript module and keeps the `import` and `export …
/// from` statements among them, and nothing else (spec PARSE-4).
pub fn read_statements(text: &str) -> Result<Vec<Import>, Broken> {
    parse(text, SourceType::jsx().with_module(true), false).map(|read| {
        read.imports
            .into_iter()
            .filter(|import| matches!(import.form, Form::Import | Form::Export | Form::ExportAll))
            .collect()
    })
}

/// Parses as a module first and then as a script.
fn read_as(text: &str, source_type: SourceType, component: bool) -> Result<Read, Broken> {
    let module = source_type.with_module(true);

    match parse(text, module, component) {
        Ok(read) => Ok(read),
        Err(broken) => parse(text, module.with_script(true), component).map_err(|_| broken),
    }
}

/// The language of a script, from its name. A type declaration file is TypeScript, and JavaScript allows
/// JSX whatever its ending.
fn source_type(path: &str) -> SourceType {
    let typescript = [".ts", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending));
    let declaration = [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|ending| path.ends_with(ending));

    if path.ends_with(".tsx") {
        SourceType::tsx()
    } else if typescript {
        SourceType::ts().with_typescript_definition(declaration)
    } else {
        SourceType::jsx()
    }
}

/// Parses once with the source type given. In a Vue component's block, the default export is the
/// component itself, so what it names is used and not passed on (spec USE-4).
fn parse(text: &str, source_type: SourceType, component: bool) -> Result<Read, Broken> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
    let lines = Lines::of(text);

    if let Some(error) = parsed.diagnostics.iter().next() {
        let line = error.labels.first().map(|label| lines.at(label.offset()));

        return Err(Broken {
            line,
            message: error.message.to_string(),
        });
    }

    if parsed.fatal_error {
        return Err(Broken {
            line: None,
            message: "the parser stopped".to_owned(),
        });
    }

    // The binding analysis is oxc's: each reference tied to the binding it names (spec USE-1).
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let mut finder = Finder {
        lines: &lines,
        semantic: &semantic,
        component,
        imports: Vec::new(),
        computed: 0,
        bound: Vec::new(),
    };

    finder.visit_program(&parsed.program);

    let exports = finder.exports(&parsed.program);

    finder.used();

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
        exports,
        named: Vec::new(),
    })
}

/// Where each line of a text begins, to turn an offset into a line and a column.
struct Lines<'t> {
    /// The text.
    text: &'t str,
    /// The offset each line begins at.
    starts: Vec<usize>,
}

impl<'t> Lines<'t> {
    /// The starts of the lines of a text.
    fn of(text: &'t str) -> Self {
        Self {
            text,
            starts: std::iter::once(0)
                .chain(text.match_indices('\n').map(|(at, _)| at + 1))
                .collect(),
        }
    }

    /// The line an offset lies on, counted from one.
    fn at(&self, offset: u32) -> usize {
        let offset = usize::try_from(offset).unwrap_or(usize::MAX);

        self.starts.partition_point(|start| *start <= offset)
    }

    /// The line an offset lies on and its column there, each counted from one, the column in UTF-16
    /// code units, as SARIF counts one.
    fn place(&self, offset: u32) -> (usize, usize) {
        let line = self.at(offset);
        let start = self.starts[line - 1];
        let before = usize::try_from(offset)
            .ok()
            .and_then(|offset| self.text.get(start..offset))
            .unwrap_or_default();

        (line, before.encode_utf16().count() + 1)
    }
}

/// Walks a program and collects its imports, with the names each brings in.
struct Finder<'s, 'a> {
    lines: &'s Lines<'s>,
    semantic: &'s Semantic<'a>,
    component: bool,
    imports: Vec<Import>,
    computed: usize,
    /// The binding each name of an import makes: its symbol, and the places of the import and of the
    /// name in their lists.
    bound: Vec<(SymbolId, usize, usize)>,
}

impl<'a> Finder<'_, 'a> {
    /// Records one import and returns its place in the list.
    fn found(&mut self, specifier: String, form: Form, type_only: bool, span: Span) -> usize {
        self.imports.push(Import {
            specifier,
            form,
            type_only,
            line: self.lines.at(span.start),
            names: Vec::new(),
            uses: Vec::new(),
        });

        self.imports.len() - 1
    }

    /// Records a name the import at the place given brings in, with the binding it makes, or the
    /// name an `export … from` offers it under.
    fn brings(
        &mut self,
        import: usize,
        name: String,
        local: Option<&BindingIdentifier<'a>>,
        exported: Option<String>,
    ) {
        let names = &mut self.imports[import].names;

        if let Some(local) = local {
            self.bound.push((local.symbol_id(), import, names.len()));
        }

        names.push(Name {
            name,
            local: local.map(|local| local.name.to_string()),
            exported: exported.into_iter().collect(),
        });
    }

    /// Records that the import at the place given uses every export of its target, from its own line.
    fn uses_every(&mut self, import: usize) {
        let line = self.imports[import].line;

        self.imports[import].uses.push(Used { name: None, line });
    }

    /// The text of a string literal, as JavaScript reads it.
    fn text(literal: &StringLiteral<'_>) -> String {
        literal.value.display().to_string()
    }

    /// The text of an argument that is a string or a template with no substitution, or nothing.
    fn literal(expression: &Expression<'_>) -> Option<String> {
        match expression {
            Expression::StringLiteral(literal) => Some(Self::text(literal)),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
                .quasis
                .first()
                .map(|quasi| match quasi.value.cooked {
                    Some(cooked) => cooked.display().to_string(),
                    None => quasi.value.raw.to_string(),
                }),
            _ => None,
        }
    }

    /// The name an `import` or `export` statement writes for an export.
    fn written(name: &ModuleExportName<'_>) -> String {
        match name {
            ModuleExportName::IdentifierName(identifier) => identifier.name.to_string(),
            ModuleExportName::IdentifierReference(identifier) => identifier.name.to_string(),
            ModuleExportName::StringLiteral(literal) => Self::text(literal),
        }
    }

    /// The import and the name a symbol is the binding of, where it is one.
    fn binding(&self, symbol: SymbolId) -> Option<(usize, usize)> {
        self.bound
            .iter()
            .find(|(bound, _, _)| *bound == symbol)
            .map(|(_, import, name)| (*import, *name))
    }

    /// Whether a reference is only the naming of a binding in an `export` statement, which passes a
    /// use on and is not one (spec USE-2). A component's default export is the component, so naming a
    /// binding there uses it (spec USE-4).
    fn exporting(&self, reference: NodeId) -> bool {
        match self.semantic.nodes().parent_kind(reference) {
            AstKind::ExportSpecifier(_) => true,
            AstKind::ExportDefaultDeclaration(_) => !self.component,
            _ => false,
        }
    }

    /// The names the script's own `export` statements offer, each once. A name that is an import's
    /// binding is passed on, and recorded on the import (spec USE-1, USE-2a). Only the statements at
    /// the top of the script are a module's exports.
    fn exports(&mut self, program: &Program<'a>) -> Vec<Export> {
        let mut found: Vec<Export> = Vec::new();

        for statement in &program.body {
            match statement {
                Statement::ExportDeclaration(export) => {
                    let mut declared = Vec::new();

                    Self::declared(&export.declaration, &mut declared);

                    for identifier in declared {
                        self.offered(
                            identifier.name.to_string(),
                            identifier.span,
                            Some(identifier.symbol_id()),
                            &mut found,
                        );
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    for specifier in &export.specifiers {
                        if let ModuleExportName::IdentifierReference(local) = &specifier.local {
                            let symbol = self
                                .semantic
                                .scoping()
                                .get_reference(local.reference_id())
                                .symbol_id();

                            self.offered(
                                Self::written(&specifier.exported),
                                specifier.exported.span(),
                                symbol,
                                &mut found,
                            );
                        }
                    }
                }
                Statement::ExportDefaultDeclaration(export) => {
                    let symbol = match &export.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                            function.id.as_ref().map(BindingIdentifier::symbol_id)
                        }
                        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                            class.id.as_ref().map(BindingIdentifier::symbol_id)
                        }
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
                            Some(interface.id.symbol_id())
                        }
                        ExportDefaultDeclarationKind::Identifier(identifier) => self
                            .semantic
                            .scoping()
                            .get_reference(identifier.reference_id())
                            .symbol_id(),
                        _ => None,
                    };

                    self.offered(
                        DEFAULT.to_owned(),
                        export.declaration.span(),
                        symbol,
                        &mut found,
                    );
                }
                _ => {}
            }
        }

        found
    }

    /// The bindings a declaration makes, which an `export` before it offers by their names.
    fn declared<'d>(declaration: &'d Declaration<'a>, found: &mut Vec<&'d BindingIdentifier<'a>>) {
        match declaration {
            Declaration::VariableDeclaration(variables) => {
                for declarator in &variables.declarations {
                    Self::patterned(&declarator.id, found);
                }
            }
            Declaration::FunctionDeclaration(function) => found.extend(&function.id),
            Declaration::ClassDeclaration(class) => found.extend(&class.id),
            Declaration::TSTypeAliasDeclaration(alias) => found.push(&alias.id),
            Declaration::TSInterfaceDeclaration(interface) => found.push(&interface.id),
            Declaration::TSEnumDeclaration(enumeration) => found.push(&enumeration.id),
            Declaration::TSNamespaceDeclaration(namespace) => found.push(&namespace.id),
            Declaration::TSImportEqualsDeclaration(import) => found.push(&import.id),
            Declaration::TSExternalModuleDeclaration(_) | Declaration::TSGlobalDeclaration(_) => {}
        }
    }

    /// The bindings a pattern makes: a name, or each name a destructuring holds.
    fn patterned<'d>(pattern: &'d BindingPattern<'a>, found: &mut Vec<&'d BindingIdentifier<'a>>) {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => found.push(identifier),
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    Self::patterned(&property.value, found);
                }

                if let Some(rest) = &object.rest {
                    Self::patterned(&rest.argument, found);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    Self::patterned(element, found);
                }

                if let Some(rest) = &array.rest {
                    Self::patterned(&rest.argument, found);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                Self::patterned(&assignment.left, found);
            }
        }
    }

    /// Records one name an `export` statement offers. The binding of an import is passed on under the
    /// name; anything else is the script's own export, a value unless its binding is a type alone, and
    /// one export when the name is offered more than once, as overloads and merged declarations are.
    fn offered(
        &mut self,
        name: String,
        span: Span,
        symbol: Option<SymbolId>,
        found: &mut Vec<Export>,
    ) {
        let passed = symbol
            .and_then(|symbol| self.binding(symbol))
            .filter(|_| !(self.component && name == DEFAULT));

        if let Some((import, bound)) = passed {
            self.imports[import].names[bound].exported.push(name);

            return;
        }

        let scoping = self.semantic.scoping();
        let is_type = symbol.is_some_and(|symbol| !scoping.symbol_flags(symbol).is_value());
        let used_in_file = symbol.is_some_and(|symbol| {
            scoping
                .get_resolved_references(symbol)
                .any(|reference| !self.exporting(reference.node_id()))
        });

        match found.iter_mut().find(|export| export.name == name) {
            Some(export) => {
                export.is_type &= is_type;
                export.used_in_file |= used_in_file;
            }
            None => {
                let (line, column) = self.lines.place(span.start);

                found.push(Export {
                    name,
                    is_type,
                    line,
                    column,
                    used_in_file,
                });
            }
        }
    }

    /// Records what each import's bindings are used for: a named or default binding uses its export
    /// where the script refers to it, in code or in a type; a namespace's binding uses the member read
    /// off it, and every export where it is referred to in any other way (spec USE-2).
    fn used(&mut self) {
        let scoping = self.semantic.scoping();
        let nodes = self.semantic.nodes();

        for (symbol, import, name) in self.bound.clone() {
            // `import name = require('…')` uses every export already, whatever is read off the name.
            if self.imports[import].form == Form::ImportEquals {
                continue;
            }

            let imported = self.imports[import].names[name].name.clone();
            let mut uses: Vec<Used> = Vec::new();

            for reference in scoping.get_resolved_references(symbol) {
                let node = reference.node_id();

                if self.exporting(node) {
                    continue;
                }

                let used = if imported == EVERY {
                    member(nodes.parent_kind(node), node)
                } else {
                    Some(imported.clone())
                };

                if uses.iter().all(|earlier| earlier.name != used) {
                    uses.push(Used {
                        name: used,
                        line: self.lines.at(nodes.kind(node).span().start),
                    });
                }
            }

            self.imports[import].uses.extend(uses);
        }
    }
}

/// The member a reference to a namespace reads off it, where its parent reads one by a name the
/// script writes: `ns.name`, `ns["name"]`, `ns.Name` in a type, or `<ns.Name />` (spec USE-2).
fn member(parent: AstKind<'_>, reference: NodeId) -> Option<String> {
    match parent {
        AstKind::StaticMemberExpression(member) => {
            is_reference(&member.object, reference).then(|| member.property.name.to_string())
        }
        AstKind::ComputedMemberExpression(member) => is_reference(&member.object, reference)
            .then(|| Finder::literal(&member.expression))
            .flatten(),
        AstKind::TSQualifiedName(qualified) => match &qualified.left {
            TSTypeName::IdentifierReference(identifier)
                if identifier.node_id.get() == reference =>
            {
                Some(qualified.right.name.to_string())
            }
            _ => None,
        },
        AstKind::JSXMemberExpression(member) => match &member.object {
            JSXMemberExpressionObject::IdentifierReference(identifier)
                if identifier.node_id.get() == reference =>
            {
                Some(member.property.name.to_string())
            }
            _ => None,
        },
        _ => None,
    }
}

/// Whether an expression is the reference with the node given.
fn is_reference(expression: &Expression<'_>, reference: NodeId) -> bool {
    matches!(expression, Expression::Identifier(identifier) if identifier.node_id.get() == reference)
}

impl<'a> Visit<'a> for Finder<'_, 'a> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        // `import { type A, type B }` brings in types alone, as `import type` does.
        let types_alone = declaration.specifiers.as_ref().is_some_and(|specifiers| {
            !specifiers.is_empty()
                && specifiers.iter().all(|specifier| {
                    matches!(
                        specifier,
                        ImportDeclarationSpecifier::ImportSpecifier(named)
                            if named.import_kind.is_type()
                    )
                })
        });
        let import = self.found(
            Self::text(&declaration.source),
            Form::Import,
            declaration.import_kind.is_type() || types_alone,
            declaration.span,
        );

        for specifier in declaration.specifiers.iter().flatten() {
            let name = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    Self::written(&specifier.imported)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => DEFAULT.to_owned(),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => EVERY.to_owned(),
            };

            self.brings(import, name, Some(specifier.local()), None);
        }

        walk::walk_import_declaration(self, declaration);
    }

    fn visit_export_from_declaration(&mut self, declaration: &ExportFromDeclaration<'a>) {
        // `export { type A, type B } from` passes types alone on, as `export type` does.
        let types_alone = !declaration.specifiers.is_empty()
            && declaration
                .specifiers
                .iter()
                .all(|specifier| specifier.export_kind.is_type());
        let import = self.found(
            Self::text(&declaration.source),
            Form::Export,
            declaration.export_kind.is_type() || types_alone,
            declaration.span,
        );

        for specifier in &declaration.specifiers {
            self.brings(
                import,
                Self::written(&specifier.local),
                None,
                Some(Self::written(&specifier.exported)),
            );
        }

        walk::walk_export_from_declaration(self, declaration);
    }

    fn visit_export_all_declaration(&mut self, declaration: &ExportAllDeclaration<'a>) {
        let import = self.found(
            Self::text(&declaration.source),
            Form::ExportAll,
            declaration.export_kind.is_type(),
            declaration.span,
        );

        self.brings(
            import,
            EVERY.to_owned(),
            None,
            declaration.exported.as_ref().map(Self::written),
        );

        walk::walk_export_all_declaration(self, declaration);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        if let TSModuleReference::ExternalModuleReference(reference) = &declaration.module_reference
        {
            let import = self.found(
                Self::text(&reference.expression),
                Form::ImportEquals,
                declaration.import_kind.is_type(),
                declaration.span,
            );

            self.brings(import, EVERY.to_owned(), Some(&declaration.id), None);
            self.uses_every(import);
        }

        walk::walk_ts_import_equals_declaration(self, declaration);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        match Self::literal(&expression.source) {
            Some(specifier) => {
                let import = self.found(specifier, Form::Dynamic, false, expression.span);

                self.uses_every(import);
            }
            None => self.computed += 1,
        }

        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && callee.name == "require"
        {
            match call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(Self::literal)
            {
                Some(specifier) => {
                    let import = self.found(specifier, Form::Require, false, call.span);

                    self.uses_every(import);
                }
                None => self.computed += 1,
            }
        }

        walk::walk_call_expression(self, call);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        let found = self.found(
            Self::text(&import.source),
            Form::TypeImport,
            true,
            import.span,
        );
        // `import('./x').A.B` reads `A` off the file; with nothing read off it, it is the whole file.
        let mut qualifier = import.qualifier.as_ref();

        while let Some(TSImportTypeQualifier::QualifiedName(qualified)) = qualifier {
            qualifier = Some(&qualified.left);
        }

        match qualifier {
            Some(TSImportTypeQualifier::Identifier(identifier)) => {
                let line = self.imports[found].line;

                self.brings(found, identifier.name.to_string(), None, None);
                self.imports[found].uses.push(Used {
                    name: Some(identifier.name.to_string()),
                    line,
                });
            }
            _ => self.uses_every(found),
        }

        walk::walk_ts_import_type(self, import);
    }
}
