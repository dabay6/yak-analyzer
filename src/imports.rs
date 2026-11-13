//! What an import is: parsing a script and finding every import in it (spec PARSE-1 to PARSE-4, IMP-1
//! to IMP-5).

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ExportAllDeclaration, ExportFromDeclaration, Expression,
    ImportDeclaration, ImportExpression, StringLiteral, TSImportEqualsDeclaration, TSImportType,
    TSModuleReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

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

/// One import.
#[derive(Debug, PartialEq, Eq)]
pub struct Import {
    /// The specifier as it is written.
    pub specifier: String,
    /// How it is written.
    pub form: Form,
    /// Whether it brings in types alone: `import type`, `export type`, or a type import.
    pub type_only: bool,
    /// The line it begins on, counted from one.
    pub line: usize,
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

/// What reading a script found.
#[derive(Debug)]
pub struct Read {
    /// Its imports, in the order they are written.
    pub imports: Vec<Import>,
    /// How many calls of `import(…)` or `require(…)` have an argument that cannot be followed
    /// (spec IMP-2).
    pub computed: usize,
}

/// Why a script did not parse: the line of the first syntax error, where there is one, and its message.
#[derive(Debug)]
pub struct Broken {
    /// The line, counted from one.
    pub line: Option<usize>,
    /// What the parser said.
    pub message: String,
}

/// Parses a script, as a module first and then as a script, and finds its imports (spec PARSE-1).
pub fn read(path: &str, text: &str) -> Result<Read, Broken> {
    read_as(text, source_type(path))
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

    Some(read_as(text, source_type))
}

/// Parses statements of an MDX document as a JavaScript module and keeps the `import` and `export …
/// from` statements among them, and nothing else (spec PARSE-4).
pub fn read_statements(text: &str) -> Result<Vec<Import>, Broken> {
    parse(text, SourceType::jsx().with_module(true)).map(|read| {
        read.imports
            .into_iter()
            .filter(|import| matches!(import.form, Form::Import | Form::Export | Form::ExportAll))
            .collect()
    })
}

/// Parses as a module first and then as a script.
fn read_as(text: &str, source_type: SourceType) -> Result<Read, Broken> {
    let module = source_type.with_module(true);

    match parse(text, module) {
        Ok(read) => Ok(read),
        Err(broken) => parse(text, module.with_script(true)).map_err(|_| broken),
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

/// Parses once with the source type given.
fn parse(text: &str, source_type: SourceType) -> Result<Read, Broken> {
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

    let mut finder = Finder {
        lines: &lines,
        imports: Vec::new(),
        computed: 0,
    };

    finder.visit_program(&parsed.program);

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
    })
}

/// Where each line of a text begins, to turn an offset into a line.
struct Lines(Vec<usize>);

impl Lines {
    /// The starts of the lines of a text.
    fn of(text: &str) -> Self {
        Self(
            std::iter::once(0)
                .chain(text.match_indices('\n').map(|(at, _)| at + 1))
                .collect(),
        )
    }

    /// The line an offset lies on, counted from one.
    fn at(&self, offset: u32) -> usize {
        let offset = usize::try_from(offset).unwrap_or(usize::MAX);

        self.0.partition_point(|start| *start <= offset)
    }
}

/// Walks a program and collects its imports.
struct Finder<'l> {
    lines: &'l Lines,
    imports: Vec<Import>,
    computed: usize,
}

impl Finder<'_> {
    /// Records one import.
    fn found(&mut self, specifier: String, form: Form, type_only: bool, span: Span) {
        self.imports.push(Import {
            specifier,
            form,
            type_only,
            line: self.lines.at(span.start),
        });
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
}

impl<'a> Visit<'a> for Finder<'_> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        self.found(
            Self::text(&declaration.source),
            Form::Import,
            declaration.import_kind.is_type(),
            declaration.span,
        );
        walk::walk_import_declaration(self, declaration);
    }

    fn visit_export_from_declaration(&mut self, declaration: &ExportFromDeclaration<'a>) {
        self.found(
            Self::text(&declaration.source),
            Form::Export,
            declaration.export_kind.is_type(),
            declaration.span,
        );
        walk::walk_export_from_declaration(self, declaration);
    }

    fn visit_export_all_declaration(&mut self, declaration: &ExportAllDeclaration<'a>) {
        self.found(
            Self::text(&declaration.source),
            Form::ExportAll,
            declaration.export_kind.is_type(),
            declaration.span,
        );
        walk::walk_export_all_declaration(self, declaration);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        if let TSModuleReference::ExternalModuleReference(reference) = &declaration.module_reference
        {
            self.found(
                Self::text(&reference.expression),
                Form::ImportEquals,
                declaration.import_kind.is_type(),
                declaration.span,
            );
        }

        walk::walk_ts_import_equals_declaration(self, declaration);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        match Self::literal(&expression.source) {
            Some(specifier) => self.found(specifier, Form::Dynamic, false, expression.span),
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
                Some(specifier) => self.found(specifier, Form::Require, false, call.span),
                None => self.computed += 1,
            }
        }

        walk::walk_call_expression(self, call);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        self.found(
            Self::text(&import.source),
            Form::TypeImport,
            true,
            import.span,
        );
        walk::walk_ts_import_type(self, import);
    }
}
