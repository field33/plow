use crate::lint::{
    common_error_literals::NO_ROOT_PREFIX, lint_failure, lint_success, Lint, LintResult,
};
use harriet::{Directive, Item, Statement, TurtleDocument};
use regex::Regex;

#[derive(Debug, Default)]
pub struct RootPrefixMatchesPattern;

impl Lint for RootPrefixMatchesPattern {
    fn short_description(&self) -> &str {
        "Check if the value for `@prefix :` matches the pattern `http://field33.com/ontologies/ONTOLOGY_NAME/`"
    }

    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let mut root_prefix_directive = None;
        for item in &document.items {
            if let Item::Statement(Statement::Directive(Directive::Prefix(directive))) = item {
                if directive.prefix.is_none() {
                    root_prefix_directive = Some(directive);
                }
            }
        }

        if let Some(root_prefix_directive) = root_prefix_directive {
            #[allow(clippy::unwrap_used)]
            let re = Regex::new("http://field33.com/ontologies/([a-zA-Z_]*)/$").unwrap();
            if re.is_match(&root_prefix_directive.iri.iri) {
                return lint_success!(
                    "ontology root prefix directive (`@prefix :`) matches expected pattern"
                );
            }
            return lint_failure!("ontology root prefix directive (`@prefix :`) does not match expected pattern (`http://field33.com/ontologies/ONTOLOGY_NAME/`)");
        }
        lint_failure!(NO_ROOT_PREFIX)
    }
}
