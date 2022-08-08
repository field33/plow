//! This lint is not made to be used directly but directed to be used in `HasAtLeastOneValidLicenseAnnotation` lint.

use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    helpers::catch_single_annotations_which_must_exist,
    lint_failure, lint_success, Lint, LintResult,
};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::REGISTRY_LICENSE_SPDX;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:licenseSPDX`";
/// Ensures that a value for `registry:licenseSPDX` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct ExistsRegistryLicenseSPDX;

impl Lint for ExistsRegistryLicenseSPDX {
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:licenseSPDX`"
    }
    /// Lints for the existence of `registry:licenseSPDX` annotation only without validation.
    /// This lint is not made to be used directly but directed to be used in `HasAtLeastOneValidLicenseAnnotation` lint.
    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        if let Ok(rdf_graph) = document_to_graph(document) {
            if let Some(root_prefix) = get_root_prefix(document) {
                let graph = rdf_graph.borrow();
                // We explicitly pass valid data, unwrap is safe here.
                #[allow(clippy::unwrap_used)]
                let annotations = graph
                    .statements()
                    .filter(|statement| {
                        statement.subject()
                            == &rdf_factory
                                .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                            && statement.predicate()
                                == &RDFTK_IRI::from_str(REGISTRY_LICENSE_SPDX).unwrap().into()
                    })
                    .collect::<HashSet<_>>();

                if let Some(failure) =
                    catch_single_annotations_which_must_exist(&annotations, RELATED_FIELD)
                {
                    return failure;
                }

                lint_success!(format!("{RELATED_FIELD} exists."))
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
