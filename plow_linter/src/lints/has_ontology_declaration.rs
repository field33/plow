use crate::lint::common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR};
use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::{OWL_ONTOLOGY, RDF_TYPE};
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct HasOntologyDeclaration;

impl Lint for HasOntologyDeclaration {
    fn short_description(&self) -> &str {
        "Check that a ontology is declared in the file (matching the \":\" @prefix)"
    }

    /// Checks whether an ontology has been declared in the document.
    /// Currently relies on the `:` prefix to be defined in the file.
    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        if let Ok(rdf_graph) = document_to_graph(document) {
            if let Some(root_prefix) = get_root_prefix(document) {
                // We explicitly pass valid data, unwrap is safe here.
                #[allow(clippy::unwrap_used)]
                let has_exact_ontology_declaration = rdf_graph
                    .borrow()
                    .matches(
                        Some(
                            &rdf_factory
                                .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into()),
                        ),
                        Some(&RDFTK_IRI::from_str(RDF_TYPE).unwrap().into()),
                        Some(
                            &rdf_factory
                                .named_object(RDFTK_IRI::from_str(OWL_ONTOLOGY).unwrap().into()),
                        ),
                    )
                    .len()
                    == 1;

                // We explicitly pass valid data, unwrap is safe here.
                #[allow(clippy::unwrap_used)]
                if !has_exact_ontology_declaration {
                    let rdf_graph_borrow = rdf_graph.borrow();
                    let all_ontology_declarations = rdf_graph_borrow
                        .statements()
                        .filter(|statement| {
                            statement.predicate() == &RDFTK_IRI::from_str(RDF_TYPE).unwrap().into()
                                && statement.object()
                                    == &rdf_factory.named_object(
                                        RDFTK_IRI::from_str(OWL_ONTOLOGY).unwrap().into(),
                                    )
                        })
                        .collect::<HashSet<_>>();

                    let mut error_messages =
                        vec!["Unable to find ontology declaration.".to_owned()];
                    for ontology_declaration in all_ontology_declarations {
                        let ontology_iri = ontology_declaration.subject().to_string();
                        error_messages.push(format!("Found ontology declaration for `{ontology_iri}`. Maybe there is a typo, or no trailing slash?"));
                    }
                    return LintResult::Failure(error_messages);
                }

                lint_success!("Ontology declaration found.")
            } else {
                return lint_failure!(NO_ROOT_PREFIX);
            }
        } else {
            return lint_failure!(RDF_GRAPH_PARSE_ERROR);
        }
    }
}
