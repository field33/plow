use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    helpers::{
        catch_single_annotations_which_must_exist, fail_if_contains_inappropriate_word,
        literal_has_language_tag_and_it_is_english,
    },
    lint_failure, lint_success, Lint, LintResult,
};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::RDFS_COMMENT;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`rdfs:comment`";
/// An approximate character count an A4 page can handle.
const RDFS_COMMENT_MANIFEST_CONTEXT_MAX_ALLOWED_CHAR_COUNT: usize = 1800;
/// Ensures that a value for `rdfs:comment` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRdfsCommentManifestContext;

impl Lint for HasRdfsCommentManifestContext {
    fn short_description(&self) -> &str {
        "Check that the ontology is annotated with a value for `rdfs:comment`"
    }
    /// Lints for the existence of `rdfs:comment` and its validity.
    /// The meaning of this tag is the long description of the ontology.
    /// Max character count is set to 1800, as this is the approximate character count an A4 page can handle.
    /// This annotation requires a language tag present and profanity filter is applied for content tagged with am English language tag.
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
                                == &RDFTK_IRI::from_str(RDFS_COMMENT).unwrap().into()
                    })
                    .collect::<HashSet<_>>();

                if let Some(failure) =
                    catch_single_annotations_which_must_exist(&annotations, RELATED_FIELD)
                {
                    return failure;
                }

                // Only take the first comment, as we only want to lint the first one.
                // The others we ignore because their meaning is different.
                #[allow(clippy::unwrap_used)]
                let annotation = annotations.iter().next().unwrap();
                let lint_prefix = format!("The value of {RELATED_FIELD},");
                let result = annotation.object().as_literal().map_or_else(
                    || lint_failure!(format!("{lint_prefix} is not a literal.")),
                    |literal| {
                        let long_description_raw = literal.lexical_form().trim();
                        if long_description_raw.chars().count() > RDFS_COMMENT_MANIFEST_CONTEXT_MAX_ALLOWED_CHAR_COUNT {
                            return lint_failure!(format!("{lint_prefix} allows a maximum of {RDFS_COMMENT_MANIFEST_CONTEXT_MAX_ALLOWED_CHAR_COUNT} characters."));
                        }
                        if literal_has_language_tag_and_it_is_english(literal) {
                            if let Some(failure) =
                                fail_if_contains_inappropriate_word(literal, &lint_prefix)
                            {
                                return failure;
                            }
                        }
                        if !literal.has_language() {
                            return lint_failure!(format!(
                                "{lint_prefix} should be tagged with a language tag."
                            ));
                        }
                        lint_success!(format!("{lint_prefix} is valid."))
                    },
                );
                result
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
