use crate::lint::common_error_literals::NO_ROOT_PREFIX;
use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use harriet::{Directive, Item, Statement, TurtleDocument};
use plow_package_management::metadata::get_root_prefix;

#[derive(Debug, Default)]
pub struct BaseMatchesRootPrefix;

impl Lint for BaseMatchesRootPrefix {
    fn short_description(&self) -> &str {
        "Check that the @base directive matches the value for the `:` prefix."
    }

    /// Check that the @base directive matches the value for the `:` prefix.
    fn lint(&self, document: &TurtleDocument) -> LintResult {
        if let Some(root_prefix) = get_root_prefix(document) {
            let mut base_directive = None;
            for item in &document.items {
                if let Item::Statement(Statement::Directive(Directive::Base(directive))) = item {
                    if base_directive.is_some() {
                        return LintResult::Failure(vec!["Found more than one @base directive. While it is valid Turtle to redeclare the @base throughout the file, this can easily be misused and is not supported in Field 33 ontologies".to_owned()]);
                    }
                    base_directive = Some(directive);
                }
            }
            if let Some(base_directive) = base_directive {
                let base_directive_iri = &base_directive.iri.iri;
                if *base_directive_iri == *root_prefix {
                    return lint_success!("@base directive matches `:` prefix.");
                }
                return lint_failure!(format!("@base directive (`{base_directive_iri}`) doesn't match the `:` prefix (`{root_prefix}`). Maybe there is a typo, or no trailing slash?"));
            }
            return lint_failure!("Unable to find a @base directive.");
        }
        lint_failure!(NO_ROOT_PREFIX)
    }
}
