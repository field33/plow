use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    lint_failure, lint_success, Lint, LintResult,
};
use crate::lints::get_root_prefix;
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::{
    OWL_ANNOTATION_PROPERTY, OWL_CLASS, OWL_DATA_PROPERTY, OWL_OBJECT_PROPERTY, RDFS_LABEL,
};

use rdftk_iri::IRI as RDFTK_IRI;

use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`rdfs:label`";
/// Ensures that a value for `rdfs:label` is specified in the related statements.
#[derive(Debug, Default)]
pub struct ValidRdfsLabels;

impl Lint for ValidRdfsLabels {
    fn short_description(&self) -> &str {
        "Check that the related field is annotated with a value for `rdfs:label`"
    }

    /// Every `Class`, `ObjectProperty`, `DataProperty`, `AnnotationProperty` should have an `rdfs:label` annotation
    /// `rdfs:label` annotations with a string literal should contain `@en` as a language tag
    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        if let Ok(rdf_graph) = document_to_graph(document) {
            if let Some(root_prefix) = get_root_prefix(document) {
                let graph = rdf_graph.borrow();
                // We explicitly pass valid data, unwrap is safe here.
                #[allow(clippy::unwrap_used)]
                let all_subject_iris_with_selected_owl_props = graph
                    .statements()
                    .filter(|statement| {
                        if let Some(object_iri) = statement.object().as_iri() {
                            // dbg!(&object_iri.path());
                            return object_iri == &RDFTK_IRI::from_str(OWL_CLASS).unwrap().into()
                                || object_iri
                                    == &RDFTK_IRI::from_str(OWL_OBJECT_PROPERTY).unwrap().into()
                                || object_iri
                                    == &RDFTK_IRI::from_str(OWL_DATA_PROPERTY).unwrap().into()
                                || object_iri
                                    == &RDFTK_IRI::from_str(OWL_ANNOTATION_PROPERTY)
                                        .unwrap()
                                        .into();
                        }
                        false
                    })
                    .map(|statement| statement.subject().as_iri().unwrap().clone())
                    .collect::<HashSet<_>>();

                if all_subject_iris_with_selected_owl_props.is_empty() {
                    return lint_success!(format!("No statements found with a Class, ObjectProperty, DataProperty, AnnotationProperty which needs a {RELATED_FIELD} associated with it."));
                }

                let mut failures = vec![];

                // We explicitly pass valid data, unwrap is safe here.
                #[allow(clippy::unwrap_used)]
                let statements_which_have_a_rdfs_label = graph
                    .statements()
                    .filter(|statement| {

                        // We ignore the rdfs:label statements which belong to the manifest.
                        // They are linted differently.
                        // See has_rdfs_label_manifest_context.rs
                        let root_prefix_path = rdf_factory
                            .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                            .as_iri()
                            .unwrap()
                            .path()
                            .clone()
                            .to_string();
                        let subject_path = statement.subject().as_iri().unwrap().path().to_string();
                        if root_prefix_path == subject_path {
                            return false;
                        }

                        if statement.predicate() == &RDFTK_IRI::from_str(RDFS_LABEL).unwrap().into()
                        {
                            // Statement has a `rdfs:label` predicate
                            let subject_iri = statement.subject().as_iri().unwrap().clone().to_string();
                            let common_failure_prefix: &str  = &format!("The {RELATED_FIELD} associated with the subject which has the IRI {subject_iri}");

                            // Validate those labels
                            if let Some(literal) = statement.object().as_literal() {
                                // dbg!(literal.lexical_form());
                                const ENGLISH_LANGUAGE_CODE: &str = "en";
                                if let Some(language_tag) = literal.language() {

                                    if language_tag.to_string() != ENGLISH_LANGUAGE_CODE {
                                        failures.push(format!(
                                            "{common_failure_prefix} does not have `@en` as a language tag."
                                        ));
                                    }
                                }
                                else {
                                    failures.push(format!(
                                        "{common_failure_prefix} does not have any language tag."
                                    ));
                                }
                            } else {
                                failures.push(format!(
                                    "{common_failure_prefix} is not a literal."
                                ));
                            }
                            return true;
                        }
                        false
                    })
                    .map(|statement| statement.subject().as_iri().unwrap().clone())
                    .collect::<HashSet<_>>();

                for subject_iri in all_subject_iris_with_selected_owl_props
                    .difference(&statements_which_have_a_rdfs_label)
                {
                    // Statements which need an `rdfs:label` predicate
                    let iri = subject_iri.to_string();
                    failures.push(format!(
                        "The subject with the IRI {iri} does not have an {RELATED_FIELD} associated with it."
                    ));
                }
                if !failures.is_empty() {
                    return LintResult::Failure(failures);
                }
                lint_success!(format!("Every Class, ObjectProperty, DataProperty, AnnotationProperty has an {RELATED_FIELD} annotation with a string literal and @en as a language tag."))
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
