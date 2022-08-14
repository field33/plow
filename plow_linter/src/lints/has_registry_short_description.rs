use crate::{
    lint::{
        common_error_literals::NO_ROOT_PREFIX, helpers::catch_single_annotations_which_must_exist,
        lint_failure, lint_success, Lint, LintResult,
    },
    Linter, MultiReaderRdfGraph,
};

use plow_ontology::constants::REGISTRY_SHORT_DESCRIPTION;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::str::FromStr;
use std::{any::Any, collections::HashSet};

const RELATED_FIELD: &str = "`registry:shortDescription`";
/// A sane maximum character count for registry:shortDescription. Identical to the length of a twitter tweet.
const SHORT_DESCRIPTION_MAX_ALLOWED_CHAR_COUNT: usize = 280;
/// Ensures that a value for `registry:shortDescription` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRegistryShortDescription;

impl Lint for HasRegistryShortDescription {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:shortDescription`"
    }
    /// Lints for the existence and validity of `registry:shortDescription`.
    /// Maximum 280 characters are allowed.
    /// The value should contain a language tag.
    /// Profanity filter is applied for content which are tagged with an English language tag.
    fn run(
        &self,
        Linter {
            document,
            graph: MultiReaderRdfGraph { inner: rdf_graph },
            ..
        }: &Linter,
    ) -> LintResult {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
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
                        && statement.predicate()
                            == &RDFTK_IRI::from_str(REGISTRY_SHORT_DESCRIPTION)
                                .unwrap()
                                .into()
                })
                .collect::<HashSet<_>>();

            // TODO: Ask opinions about allowing multiple license annotations.
            // Currently only a single one is allowed.
            if let Some(failure) =
                catch_single_annotations_which_must_exist(&annotations, RELATED_FIELD)
            {
                return failure;
            }

            // We know that `annotations` has at least one member here.
            #[allow(clippy::unwrap_used)]
            let annotation = annotations.iter().next().unwrap();
            let lint_prefix = format!("The value of {RELATED_FIELD},");
            let result = annotation.object().as_literal().map_or_else(
                    || lint_failure!(format!("{lint_prefix} is not a literal.")),
                    |literal| {
                        let short_description_raw = literal.lexical_form().trim();
                        if short_description_raw.chars().count() > SHORT_DESCRIPTION_MAX_ALLOWED_CHAR_COUNT {
                            return lint_failure!(format!("{lint_prefix} allows a maximum of {SHORT_DESCRIPTION_MAX_ALLOWED_CHAR_COUNT} characters."));
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
                        lint_success!(format!("{lint_prefix} is valid."))
                    },
                );
            result
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
