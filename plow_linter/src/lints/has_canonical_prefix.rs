use crate::lint::common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR};
use crate::lint::helpers::catch_single_annotations_which_must_exist;
use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::REGISTRY_CANONICAL_PREFIX;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:canonicalPrefix`";
#[derive(Debug, Default)]
pub struct HasCanonicalPrefix;

impl Lint for HasCanonicalPrefix {
    fn short_description(&self) -> &str {
        "Check that the ontology is annotated with a value for `registry:canonicalPrefix`"
    }

    /// Check that the ontology is annotated with a value for `registry:canonicalPrefix`
    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        if let Ok(rdf_graph) = document_to_graph(document) {
            let rdf_graph_borrow = rdf_graph.borrow();
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
                return lint_failure!(NO_ROOT_PREFIX);
            }
        } else {
            return lint_failure!(RDF_GRAPH_PARSE_ERROR);
        }
    }
}
