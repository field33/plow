use crate::lint::common_error_literals::NO_ROOT_PREFIX;
use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use crate::Linter;

use plow_ontology::constants::{OWL_ONTOLOGY, RDF_TYPE};
use plow_package_management::metadata::get_root_prefix;

use rdftk_iri::IRI as RDFTK_IRI;
use std::any::Any;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct HasOntologyDeclaration;

// TODO: Refactor unwraps
#[allow(clippy::unwrap_used)]
impl Lint for HasOntologyDeclaration {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that a field is declared in the file (matching the \":\" @prefix)"
    }

    /// Checks whether an ontology has been declared in the document.
    /// Currently relies on the `:` prefix to be defined in the file.
    fn run(&self, linter: &Linter) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();

        if let Some(root_prefix) = get_root_prefix(&linter.document) {
            // We explicitly pass valid data, unwrap is safe here.
            #[allow(clippy::unwrap_used)]
            let graph_borrow = linter.graph.inner.borrow();
            let has_exact_ontology_declaration = graph_borrow
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
                let all_ontology_declarations = graph_borrow
                    .statements()
                    .filter(|statement| {
                        statement.predicate() == &RDFTK_IRI::from_str(RDF_TYPE).unwrap().into()
                            && statement.object()
                                == &rdf_factory
                                    .named_object(RDFTK_IRI::from_str(OWL_ONTOLOGY).unwrap().into())
                    })
                    .collect::<HashSet<_>>();

                let mut error_messages = vec!["Unable to find ontology declaration.".to_owned()];
                for ontology_declaration in all_ontology_declarations {
                    let ontology_iri = ontology_declaration.subject().to_string();
                    error_messages.push(format!("Found ontology declaration for `{ontology_iri}`. Maybe there is a typo, or no trailing slash?"));
                }
                return LintResult::Failure(error_messages);
            }

            lint_success!("Ontology declaration found in the field.")
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
