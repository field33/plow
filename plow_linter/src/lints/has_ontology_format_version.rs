use crate::lint::common_error_literals::NO_ROOT_PREFIX;
use crate::lint::helpers::catch_single_annotations_which_must_exist;
use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use crate::{Linter, MultiReaderRdfGraph};

use plow_ontology::constants::REGISTRY_ONTOLOGY_FORMAT_VERSION;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::any::Any;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:ontologyFormatVersion`";

/// Ensures that a value for `registry:ontologyFormatVersion` is specified as annotation on the ontology.
///
/// Currently the only accepted value is `v1`.
///
/// Required to allow for evolution of the ontology format over time.
#[derive(Debug, Default)]
pub struct HasOntologyFormatVersion;

impl Lint for HasOntologyFormatVersion {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:ontologyFormatVersion`, and it is equal to an acceptable value (`v1`)."
    }

    /// Check that the ontology is annotated with a value for `registry:ontologyFormatVersion`, and it is equal to an acceptable value (`v1`).
    fn run(
        &self,
        Linter {
            document,
            graph: MultiReaderRdfGraph { inner: rdf_graph },
            ..
        }: &Linter,
    ) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        let graph_ref = rdf_graph;
        let rdf_graph_borrow = graph_ref.borrow();
        let root_prefix = get_root_prefix(document);
        if let Some(root_prefix) = root_prefix {
            // We explicitly pass valid data, unwrap is safe here.
            #[allow(clippy::unwrap_used)]
            let annotations = rdf_graph_borrow
                .statements()
                .filter(|statement| {
                    statement.subject()
                        == &rdf_factory
                            .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                        && statement.predicate()
                            == &RDFTK_IRI::from_str(REGISTRY_ONTOLOGY_FORMAT_VERSION)
                                .unwrap()
                                .into()
                })
                .collect::<HashSet<_>>();
            if let Some(failure) =
                catch_single_annotations_which_must_exist(&annotations, RELATED_FIELD)
            {
                return failure;
            }

            let lint_prefix = format!("Value for {RELATED_FIELD}");

            // We know that `annotations` has at least one member here.
            #[allow(clippy::unwrap_used)]
            let annotation = annotations.iter().next().unwrap();
            match annotation.object().as_literal() {
                None => {
                    lint_failure!(format!("{lint_prefix} is not a literal."))
                }
                Some(literal) => match literal.lexical_form().as_str() {
                    "v1" => {
                        lint_success!(format!("{lint_prefix} is equal to `v1`."))
                    }
                    other => lint_failure!(format!(
                        "{lint_prefix} is not equal to `v1` (value: `{}`).",
                        other
                    )),
                },
            }
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
