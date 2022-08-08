use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    helpers::{
        catch_single_annotations_which_must_exist, validate_semantic_version_literal,
        VersionLiteralLintFailureOrWarning,
    },
    lint_failure, lint_success, lint_warning, Lint, LintResult,
};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::REGISTRY_PACKAGE_VERSION;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:packageVersion`";
/// Ensures that a value for `registry:packageVersion` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRegistryPackageVersion;

impl Lint for HasRegistryPackageVersion {
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:packageVersion`"
    }

    /// Lints for the existence of `registry:packageVersion` and its correct format
    /// (Only simple and fully complete version strings are allowed with no prefix or suffixes. e.g. major.minor.patch)
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
                                == &RDFTK_IRI::from_str(REGISTRY_PACKAGE_VERSION)
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
                if let Some(literal) = annotation.object().as_literal() {
                    // Check if the version string is valid
                    let version = literal.lexical_form();
                    let lint_prefix = format!("The value of {RELATED_FIELD},");
                    if let Err(failure_or_warning) = validate_semantic_version_literal(version) {
                        use VersionLiteralLintFailureOrWarning::*;
                        match failure_or_warning {
                            Warning(warning) => {
                                return lint_warning!(format!("{lint_prefix} {warning}"));
                            }
                            Failure(failure) => {
                                return lint_failure!(format!("{lint_prefix} {failure}"));
                            }
                        }
                    }
                    lint_success!(format!("{lint_prefix} is valid."))
                } else {
                    lint_failure!("{lint_prefix} is not a literal.")
                }
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
