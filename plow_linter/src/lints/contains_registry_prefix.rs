use std::any::Any;

use crate::lint::{lint_success, Lint, LintResult};
use crate::Linter;
use harriet::{Directive, Statement};

#[derive(Debug, Default)]
pub struct ContainsRegistryPrefix;

impl Lint for ContainsRegistryPrefix {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check if the field contains the prefix for the REGISTRY ontology"
    }

    /// Check if ontology contains the prefix for the REGISTRY ontology.
    /// Currently required to ensure that editing operations work smoothly.
    fn run(&self, Linter { document, .. }: &Linter) -> LintResult {
        let mut owl_prefixes = vec![("registry", "http://field33.com/ontologies/REGISTRY/")];

        for statement in &document.statements {
            if let Statement::Directive(Directive::Prefix(directive)) = statement {
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
            return lint_success!("The field contains the `registry:` prefix.");
        }
        LintResult::Failure(owl_prefixes.iter().map(|(prefix, iri)| {
            format!("The field is missing a prefix directive for {prefix}: `@prefix {prefix}: <{iri}> .`")
        }).collect())
    }
}
