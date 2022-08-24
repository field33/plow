use crate::{
    lint::{
        common_error_literals::NO_ROOT_PREFIX,
        helpers::{catch_single_annotations_which_may_exist, fail_if_domain_name_is_invalid},
        lint_failure, lint_success, lint_warning, Lint, LintResult,
    },
    Linter, MultiReaderRdfGraph,
};

use field33_rdftk_iri_temporary_fork::IRI as RDFTK_IRI;
use plow_ontology::constants::REGISTRY_HOMEPAGE;
use plow_package_management::metadata::get_root_prefix;
use std::str::FromStr;
use std::{any::Any, collections::HashSet};

const RELATED_FIELD: &str = "`registry:homepage`";
/// Ensures that a value for `registry:homepage` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct ValidRegistryHomepage;

impl Lint for ValidRegistryHomepage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:homepage`"
    }

    /// Lints for the validity of `registry:homepage`.
    /// Domain validation is applied.
    fn run(
        &self,
        Linter {
            document,
            graph: MultiReaderRdfGraph { inner: rdf_graph },
            ..
        }: &Linter,
    ) -> LintResult {
        let rdf_factory = field33_rdftk_core_temporary_fork::simple::statement::statement_factory();
        if let Some(root_prefix) = get_root_prefix(document) {
            let graph_ref = rdf_graph;
            let graph = graph_ref.borrow();
            // We explicitly pass valid data, unwrap is safe here.
            #[allow(clippy::unwrap_used)]
            let annotations = graph
                .statements()
                .filter(|statement| {
                    statement.subject()
                        == &rdf_factory
                            .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                        && statement.predicate()
                            == &RDFTK_IRI::from_str(REGISTRY_HOMEPAGE).unwrap().into()
                })
                .collect::<HashSet<_>>();

            if let Some(failure) =
                catch_single_annotations_which_may_exist(&annotations, RELATED_FIELD)
            {
                return failure;
            }

            // We know that `annotations` has at least one member here.
            #[allow(clippy::unwrap_used)]
            if let Some(annotation) = annotations.iter().next() {
                let lint_prefix = format!("The value of {RELATED_FIELD}");
                if let Some(literal) = annotation.object().as_literal() {
                    if let Some(failure) = fail_if_domain_name_is_invalid(literal, RELATED_FIELD) {
                        return failure;
                    }
                    lint_success!(format!("{lint_prefix} is valid."))
                } else {
                    lint_failure!(format!("{lint_prefix} is not a literal."))
                }
            } else {
                lint_warning!(format!("It might be nice to add a {RELATED_FIELD} annotation for interested individuals to learn more about this field."))
            }
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
