use crate::{
    lint::{
        common_error_literals::NO_ROOT_PREFIX, helpers::catch_single_annotations_which_must_exist,
        lint_failure, lint_success, Lint, LintResult,
    },
    Linter, MultiReaderRdfGraph,
};

use field33_rdftk_iri_temporary_fork::IRI as RDFTK_IRI;
use plow_ontology::constants::RDFS_LABEL;
use plow_package_management::metadata::get_root_prefix;
use std::str::FromStr;
use std::{any::Any, collections::HashSet};

const RELATED_FIELD: &str = "`rdfs:label`";
/// A sane character count for a title.
const RDFS_LABEL_MANIFEST_CONTEXT_MAX_ALLOWED_CHAR_COUNT: usize = 60;
/// Ensures that a value for `rdfs:label` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRdfsLabelManifestContext;

impl Lint for HasRdfsLabelManifestContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `rdfs:label`"
    }
    /// Lints for the existence of `rdfs:label` and its validity.
    /// The meaning of this tag is the title of the ontology.
    /// Max character count is set to 60.
    /// This annotation requires a language tag present and profanity filter is applied for content tagged with am English language tag.
    fn run(
        &self,
        Linter {
            document,
            graph: MultiReaderRdfGraph { inner: rdf_graph },
            ..
        }: &Linter,
    ) -> LintResult {
        let rdf_factory = field33_rdftk_core_temporary_fork::simple::statement::statement_factory();
        if let Some(root_prefix) = get_root_prefix(document) {
            let graph_ref = rdf_graph;
            let graph = graph_ref.borrow();
            // We explicitly pass valid data, unwrap is safe here.
            #[allow(clippy::unwrap_used)]
            let annotations = graph
                .statements()
                .filter(|statement| {
                    statement.subject()
                        == &rdf_factory
                            .named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                        && statement.predicate() == &RDFTK_IRI::from_str(RDFS_LABEL).unwrap().into()
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
                        let title_raw = literal.lexical_form().trim();
                        if title_raw.chars().count() > RDFS_LABEL_MANIFEST_CONTEXT_MAX_ALLOWED_CHAR_COUNT {
                            return lint_failure!(format!("{lint_prefix} allows a maximum of {RDFS_LABEL_MANIFEST_CONTEXT_MAX_ALLOWED_CHAR_COUNT} characters."));
                        }
                        // Check for profanity. Currently not applied.
                        // if literal_has_language_tag_and_it_is_english(literal) {
                        //     if let Some(failure) =
                        //         fail_if_contains_inappropriate_word(literal, &lint_prefix)
                        //     {
                        //         return failure;
                        //     }
                        // }
                        if !literal.has_language() {
                            return lint_failure!(format!(
                                "{lint_prefix} should be tagged with a language tag."
                            ));
                        }
                        lint_success!(format!("{lint_prefix} is valid, which represents the field title."))
                    },
                );
            result
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
