pub(crate) mod common_error_literals;
pub(crate) mod helpers;

use std::any::Any;

use crate::{lints::AddPrefixes, Linter};
pub use harriet::TurtleDocument;
use rayon::prelude::{FromParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::Serialize;

/// A lint that can be applied to an ontology.
pub trait Lint {
    /// A short layman-readable description of what the lint is checking for.
    fn short_description(&self) -> &str;
    /// Checks the lint for the ontology.
    fn run(&self, linter: &Linter) -> LintResult;
    /// If possible returns fixes that can be automatically applied to the ontology to resolve the warning/failure.
    #[allow(unused_variables)]
    fn suggest_fix(&self, document: &TurtleDocument) -> Option<Vec<Fixes>> {
        None
    }
    fn as_any(&self) -> &dyn Any;
    fn can_run_in_parallel(&self) -> bool {
        true
    }
}

/// A suggested fix that can be applied to an ontology to resolve a particular issue in the ontology.
pub trait FixSuggestion {
    fn apply(&self, document: &mut TurtleDocument);
}

#[derive(Debug, Serialize, Clone)]
pub enum LintResult {
    Success(String),
    Warning(Vec<String>),
    Failure(Vec<String>),
}

// Wrapper type for lint results needed for parallel execution.
pub struct LintResults {
    pub results: Vec<LintResult>,
}

impl FromParallelIterator<Vec<LintResult>> for LintResults {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = Vec<LintResult>>,
    {
        let result_chunks: Vec<Vec<LintResult>> = par_iter.into_par_iter().collect();
        let results = result_chunks.iter().flatten().cloned().collect::<Vec<_>>();
        Self { results }
    }
}

impl LintResult {
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub const fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failure(_))
    }
}

macro_rules! lint_success {
    ($exp:expr) => {
        LintResult::Success($exp.to_owned())
    };
    ($l:literal) => {
        LintResult::Success($l.to_owned())
    };
}

macro_rules! lint_warning {
    ($( $exp:expr ),+) => {
        LintResult::Warning(vec![$( $exp.to_owned() ),+])
    };
    ($( $l:literal ),+) => {
        LintResult::Warning(vec![$( $l.to_owned() ),+])
    };
}

macro_rules! lint_failure {
    ($( $exp:expr ),+) => {
        LintResult::Failure(vec![$( $exp.to_owned() ),+])
    };
    ($( $l:literal ),+) => {
        LintResult::Failure(vec![$( $l.to_owned() ),+])
    };

}

pub(crate) use {lint_failure, lint_success, lint_warning};

#[derive(Debug, Clone)]
pub enum Fixes {
    AddPrefixes(AddPrefixes),
}

impl FixSuggestion for Fixes {
    fn apply(&self, document: &mut TurtleDocument) {
        match self {
            Self::AddPrefixes(fix) => fix.apply(document),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    // We don't explicitly need restrictive lints for tests.
    #![allow(clippy::restriction)]

    use harriet::TurtleDocument;
    use nom::error::VerboseError;

    fn parse_ontology() -> TurtleDocument<'static> {
        let ontology = r#"
        @prefix : <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix xml: <http://www.w3.org/XML/1998/namespace> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .
        @base <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> .

        <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> rdf:type owl:Ontology .
            "#;
        TurtleDocument::parse::<VerboseError<&str>>(ontology)
            .unwrap()
            .1
    }

    fn parse_ontology_broken() -> TurtleDocument<'static> {
        let ontology = r#"
        @prefix : <http://field33.com/ontologies/some_unexpected_path/EXAMPLE_ONTOLOGY/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix xml: <http://www.w3.org/XML/1998/namespace> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .
        @prefix field33_virtual: <http://field33.com/ontologies/VIRTUAL/> .
        @base <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> .

        <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> rdf:type owl:Ontology .

        <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> <field33_virtual:ontologyFormatVersion> "v1" .
        <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> <http://field33.com/ontologies/VIRTUAL/canonicalPrefix> "EXAMPLE_ONTOLOGY" .
            "#;
        TurtleDocument::parse::<VerboseError<&str>>(ontology)
            .unwrap()
            .1
    }
}
