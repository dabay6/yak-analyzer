//! What `audit` reports: the rules a finding can be of (spec REPORT-2).

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// A script or a Vue component that no entry point reaches.
    UnusedFile,
    /// A value a reachable file exports that no other file uses.
    UnusedExport,
    /// A type a reachable file exports that no other file uses.
    UnusedType,
    /// An import that leads to no file.
    UnresolvedImport,
    /// Files that import each other in a loop.
    CircularDependency,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 5] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
    ];

    /// The rule's name in the report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedFile => "unused-file",
            Self::UnusedExport => "unused-export",
            Self::UnusedType => "unused-type",
            Self::UnresolvedImport => "unresolved-import",
            Self::CircularDependency => "circular-dependency",
        }
    }

    /// What the rule finds, as one sentence.
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnusedFile => "A script or a Vue component that no entry point reaches.",
            Self::UnusedExport => "A value a file exports that no other file uses.",
            Self::UnusedType => "A type a file exports that no other file uses.",
            Self::UnresolvedImport => "An import that leads to no file.",
            Self::CircularDependency => "Files that import each other in a loop.",
        }
    }
}
