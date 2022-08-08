use crate::lint::{lint_success, Fixes, Lint, LintResult};
use crate::lints::AddPrefixes;
use harriet::{Directive, Item, Statement, TurtleDocument};

#[derive(Debug, Default)]
pub struct ContainsRegistryPrefix;

impl Lint for ContainsRegistryPrefix {
    fn short_description(&self) -> &str {
        "Check if the field contains the prefix for the REGISTRY ontology"
    }

    /// Check if ontology contains the prefix for the REGISTRY ontology.
    /// Currently required to ensure that editing operations work smoothly.
    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let mut owl_prefixes = vec![("registry", "http://field33.com/ontologies/REGISTRY/")];

        for item in &document.items {
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
            return lint_success!("The field contains the `registry:` prefix.");
        }
        LintResult::Failure(owl_prefixes.iter().map(|(prefix, iri)| {
            format!("The field is missing a prefix directive for {prefix}: `@prefix {prefix}: <{iri}> .`")
        }).collect())
    }

    fn suggest_fix(&self, document: &TurtleDocument) -> Option<Vec<Fixes>> {
        if let LintResult::Success(_) = self.lint(document) {
            return None;
        }
        Some(vec![
            Fixes::AddPrefixes(AddPrefixes::for_missing_registry()),
        ])
    }
}
