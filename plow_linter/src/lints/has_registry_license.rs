use crate::{
    lint::{
        common_error_literals::NO_ROOT_PREFIX, helpers::catch_single_annotations_which_must_exist,
        lint_failure, lint_success, Lint, LintResult,
    },
    Linter, MultiReaderRdfGraph,
};

use field33_rdftk_iri_temporary_fork::IRI as RDFTK_IRI;
use plow_ontology::constants::REGISTRY_LICENSE;
use plow_package_management::metadata::get_root_prefix;
use std::str::FromStr;
use std::{any::Any, collections::HashSet};

const RELATED_FIELD: &str = "`registry:license`";
/// A sane character count for a `registry:license` field.
const LICENSE_MAX_ALLOWED_CHAR_COUNT: usize = 10_000;
/// Ensures that a value for `registry:license` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRegistryLicense;

impl Lint for HasRegistryLicense {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:license`"
    }
    /// Lints for the existence of `registry:license` and its validity.
    /// Check <https://spdx.org/licenses> for a list of available licenses.
    /// Maximum 10000 characters are allowed for a license.
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
                            == &RDFTK_IRI::from_str(REGISTRY_LICENSE).unwrap().into()
                })
                .collect::<HashSet<_>>();

            if let Some(failure) =
                catch_single_annotations_which_must_exist(&annotations, RELATED_FIELD)
            {
                return failure;
            }

            // We know that `annotations` has at least one member here.
            #[allow(clippy::unwrap_used)]
            let annotation = annotations.iter().next().unwrap();
            let lint_prefix = format!("The value of {RELATED_FIELD},");
            let result = annotation.object().as_literal().map_or_else(
                    || lint_failure!(format!("{lint_prefix} is not a literal.")),
                    |literal| {
                  
                        let license_raw = literal.lexical_form().trim();
                        if license_raw.chars().count() > LICENSE_MAX_ALLOWED_CHAR_COUNT {
                            return lint_failure!(format!("{lint_prefix} allows a maximum of {LICENSE_MAX_ALLOWED_CHAR_COUNT} characters."));
                        }

                        // Profanity filter is off for license.
                        // if let Some(failure) =
                        //     fail_if_contains_inappropriate_word(literal, &lint_prefix)
                        // {
                        //     return failure;
                        // }
                        lint_success!(format!("{lint_prefix} is valid."))
                    },
                );
            result
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
