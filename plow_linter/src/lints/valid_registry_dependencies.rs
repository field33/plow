use crate::lint::helpers::{
    validate_namespace_and_name, validate_semantic_version_requirement_literal,
    VersionLiteralLintFailureOrWarning,
};
use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    lint_failure, lint_success, Lint, LintResult,
};
use plow_graphify::document_to_graph;
use harriet::TurtleDocument;
use plow_ontology::constants::REGISTRY_DEPENDENCY;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;

use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:dependency`";
/// Ensures that values for `registry:dependency` fields are valid.
#[derive(Debug, Default)]
pub struct ValidRegistryDependencies;

impl Lint for ValidRegistryDependencies {
    fn short_description(&self) -> &str {
        "Check the validity of the values associated to `registry:dependency` fields."
    }
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
                                == &RDFTK_IRI::from_str(REGISTRY_DEPENDENCY).unwrap().into()
                    })
                    .collect::<HashSet<_>>();

                if annotations.is_empty() {
                    // Nothing to lint
                    return lint_success!("Your file does not have any dependencies.");
                }

                let mut failures = vec![];
                let mut warnings = vec![];
                for annotation in &annotations {
                    if let Some(literal) = annotation.object().as_literal() {
                        let dependency_literal = literal.lexical_form();
                        let lint_prefix = format!(
                            "The {RELATED_FIELD} with stated dependency {dependency_literal},"
                        );

                        let (dependency_name, mut version) =
                            literal.lexical_form().split(' ').enumerate().fold(
                                (String::default(), String::default()),
                                |(mut name, mut version), (index, part)| {
                                    if index > 0 {
                                        version.push(' ');
                                        version.push_str(part);
                                    } else {
                                        name = part.to_owned();
                                    }
                                    (name, version)
                                },
                            );
                        version.remove(0);

                        if dependency_name.is_empty() || version.is_empty() {
                            failures.push(format!(
                                "{lint_prefix} should have both name and version separated by a single space."
                            ));
                            continue;
                        }

                        if let Err(failure) = validate_namespace_and_name(&dependency_name) {
                            failures.push(format!("{lint_prefix} {failure}"));
                        }

                        if let Err(failures_or_warnings) =
                            validate_semantic_version_requirement_literal(&version)
                        {
                            use VersionLiteralLintFailureOrWarning::*;
                            for failure_or_warning in failures_or_warnings {
                                match failure_or_warning {
                                    Warning(warning) => {
                                        warnings.push(format!("{lint_prefix} {warning}"));
                                    }
                                    Failure(failure) => {
                                        failures.push(format!("{lint_prefix} {failure}"));
                                    }
                                }
                            }
                        }
                    } else {
                        failures.push(format!(
                            "The value for {RELATED_FIELD} with stated dependency should be a literal."
                        ));
                    }
                }
                if !failures.is_empty() {
                    return LintResult::Failure(failures);
                }
                if !warnings.is_empty() {
                    return LintResult::Warning(warnings);
                }
                lint_success!("All values attached to {RELATED_FIELD} fields are valid.")
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
