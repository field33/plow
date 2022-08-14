use crate::lint::common_error_literals::NO_ROOT_PREFIX;
use crate::lint::helpers::catch_single_annotations_which_must_exist;
use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use crate::{Linter, MultiReaderRdfGraph};
use plow_ontology::constants::REGISTRY_CANONICAL_PREFIX;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::any::Any;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:canonicalPrefix`";
#[derive(Debug, Default)]
pub struct HasCanonicalPrefix;

impl Lint for HasCanonicalPrefix {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:canonicalPrefix`"
    }

    /// Check that the ontology is annotated with a value for `registry:canonicalPrefix`
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
        if let Some(root_prefix) = get_root_prefix(document) {
            // We explicitly pass valid data, unwrap is safe here.
            #[allow(clippy::unwrap_used)]
            let annotations = rdf_graph_borrow
                .statements()
                .filter(|statement| {
                    statement.subject()
                        == &rdf_factory
                            .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                        && statement.predicate()
                            == &RDFTK_IRI::from_str(REGISTRY_CANONICAL_PREFIX)
                                .unwrap()
                                .into()
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
            if annotation.object().as_literal().is_none() {
                return lint_failure!(format!("Value for {RELATED_FIELD} is not a literal."));
            }

            lint_success!(format!("Value for {RELATED_FIELD} is present."))
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
