use std::any::Any;

use crate::{
    lint::{Lint, LintResult},
    Linter,
};
use harriet::{Directive, Item, Statement};

/// Ensures that all the Turtle @prefix directives well-known to the OWL2 standard are present.
///
/// See `Table 2` under <https://www.w3.org/TR/2012/REC-owl2-syntax-20121211/#IRIs> for the list of
/// well-known prefixes.
///
/// This lint is required to ensure that common editing operations on the ontology work smoothly,
/// as well as to ensure interoperability with Protege and common OWL2 tooling.
#[derive(Debug, Default)]
pub struct ContainsOWLPrefixes;

impl Lint for ContainsOWLPrefixes {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check if the field contains all important OWL prefixes"
    }

    /// Check if ontology contains all important OWL prefixes
    fn run(&self, linter: &Linter) -> LintResult {
        let mut owl_prefixes = vec![
            ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
            ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
            ("xml", "http://www.w3.org/XML/1998/namespace"),
            ("xsd", "http://www.w3.org/2001/XMLSchema#"),
            ("owl", "http://www.w3.org/2002/07/owl#"),
        ];

        for item in &linter.document.items {
            if let Item::Statement(Statement::Directive(Directive::Prefix(directive))) = item {
                if let Some(ref directive_prefix) = directive.prefix {
                    owl_prefixes = owl_prefixes
                        .into_iter()
                        .filter(|(prefix, _)| directive_prefix != prefix)
                        .collect();
                }
                continue;
            }
        }

        if owl_prefixes.is_empty() {
            return LintResult::Success(
                "The field contains all prefixes referenced in OWL2 standard / necessary for Protege."
                    .to_owned(),
            );
        }
        LintResult::Failure(owl_prefixes.iter().map(|(prefix, iri)| {
            format!("The field is missing a prefix directive for {prefix}: `@prefix {prefix}: <{iri}> .`", prefix = prefix, iri = iri)
        }).collect())
    }
}
