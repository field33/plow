use crate::{
    lint::{
        common_error_literals::NO_ROOT_PREFIX,
        helpers::{
            catch_single_or_multiple_annotations_which_must_exist,
            fail_if_contains_inappropriate_word, fail_if_has_language_tag,
        },
        lint_failure, lint_success, Lint, LintResult,
    },
    Linter, MultiReaderRdfGraph,
};

use field33_rdftk_iri_temporary_fork::IRI as RDFTK_IRI;
use plow_ontology::constants::REGISTRY_KEYWORD;
use plow_package_management::metadata::get_root_prefix;
use std::str::FromStr;
use std::{any::Any, collections::HashSet};

const RELATED_FIELD: &str = "`registry:keyword`";
/// Maximum count of keywords.
const MAX_KEYWORDS: usize = 5;
/// A sane character count for a single keyword.
const KEYWORD_MAX_ALLOWED_CHAR_COUNT: usize = 50;

/// Ensures that a value for `registry:keyword` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRegistryKeyword;

impl Lint for HasRegistryKeyword {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:keyword`"
    }
    /// Lints for the existence of `registry:keyword` and its validity.
    /// Keywords could be chosen freely.
    /// Maximum 5 keywords are allowed.
    /// Keywords shouldn't contain language tags.
    /// Maximum 50 characters are allowed for a keyword.
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
                        && statement.predicate()
                            == &RDFTK_IRI::from_str(REGISTRY_KEYWORD).unwrap().into()
                })
                .collect::<HashSet<_>>();

            if let Some(failure) =
                catch_single_or_multiple_annotations_which_must_exist(&annotations, RELATED_FIELD)
            {
                return failure;
            }

            let lint_prefix = format!("The value of {RELATED_FIELD},");
            if annotations.len() > MAX_KEYWORDS {
                return lint_failure!(format!(
                        "{lint_prefix} can not contain more than {MAX_KEYWORDS} keywords. Please reduce the amount of keywords.",
                    ));
            }

            // Checking for duplicate keywords are actually unnecessary. Because they're checked on parsing level.
            // But I'm keeping it here if harriet decides to allow for duplicate annotations later.
            let mut checked_literals: Vec<&str> = vec![];
            let lint_results = annotations
                    .iter()
                    .map(|annotation| {
                        annotation.object().as_literal().map_or_else(
                            || lint_failure!(format!("{lint_prefix} is not a literal.")),
                            |literal| {
                                if let Some(failure) = fail_if_has_language_tag(literal, RELATED_FIELD) {
                                    return failure;
                                }
                                if let Some(failure) = fail_if_contains_inappropriate_word(literal,&lint_prefix ) {
                                    return failure;
                                }
                                let keyword_raw = literal.lexical_form().trim();
                                if checked_literals.contains(&keyword_raw) {
                                   return lint_failure!(format!(
                                    "Each {RELATED_FIELD} value can only be used only once. There could be no duplicates."
                                   ));
                                }
                                checked_literals.push(keyword_raw);
                                if keyword_raw.chars().count() > KEYWORD_MAX_ALLOWED_CHAR_COUNT {
                                    return lint_failure!(format!("{lint_prefix} allows a maximum of {KEYWORD_MAX_ALLOWED_CHAR_COUNT} characters."));
                                }
                                lint_success!(format!("{lint_prefix} is valid."))
                            },
                        )
                    })
                    .collect::<Vec<LintResult>>();

            checked_literals.clear();
            for result in lint_results {
                if let LintResult::Failure(messages) = result {
                    return lint_failure!(format!(
                        "Some {RELATED_FIELD} annotations are invalid. More info: {}",
                        messages.join(", ")
                    ));
                }
            }
            lint_success!(format!("All {RELATED_FIELD} annotations are valid."))
        } else {
            lint_failure!(NO_ROOT_PREFIX)
        }
    }
}
