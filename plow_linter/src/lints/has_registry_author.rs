use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    helpers::{catch_single_or_multiple_annotations_which_must_exist, fail_if_contains_inappropriate_word},
    lint_failure, lint_success, Lint, LintResult,
};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::REGISTRY_AUTHOR;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:author`";
/// A sane character count for a persons name, it is a guess so might be updated later.
const AUTHOR_NAME_MAX_ALLOWED_CHAR_COUNT: usize = 50;
/// Ensures that a value for `registry:author` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRegistryAuthor;

impl Lint for HasRegistryAuthor {
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:author`"
    }
    /// Lints for the existence of `registry:author` and its validity.
    /// Max character count is set to 50.
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
                                == &RDFTK_IRI::from_str(REGISTRY_AUTHOR).unwrap().into()
                    })
                    .collect::<HashSet<_>>();

                if let Some(failure) =
                    catch_single_or_multiple_annotations_which_must_exist(&annotations, RELATED_FIELD)
                {
                    return failure;
                }

                let lint_prefix = RELATED_FIELD.to_owned();

                let lint_results = annotations.iter().map(|annotation| {
                    annotation.object().as_literal().map_or_else(
                        || lint_failure!(format!("{lint_prefix} is not a literal.")),
                        |literal| {
                            if let Some(failure) = fail_if_contains_inappropriate_word(literal,&lint_prefix ) {
                                return failure;
                            }
                            let author_literal = literal.lexical_form();
                            let name_and_email_raw = author_literal.split('<').collect::<Vec<_>>();

                            name_and_email_raw.get(0).map_or_else(|| {
                                lint_failure!(format!("{lint_prefix} is not in its right form. Example of a right form: \"An Author's Name <email@oftheauthor.com>\"."))
                            }, |maybe_name| if maybe_name.ends_with(' ') {
                                    let trimmed_maybe_name = maybe_name.trim();
                                    if trimmed_maybe_name.contains('>') || trimmed_maybe_name.contains('<')
                                    {
                                       return lint_failure!(format!("The name field in {lint_prefix} can not contain neither '<' nor '>' characters."));
                                    }
                                    if trimmed_maybe_name.chars().count()
                                        > (AUTHOR_NAME_MAX_ALLOWED_CHAR_COUNT + 1)
                                    {
                                        return lint_failure!(format!("The name field in {lint_prefix} allows a maximum of {AUTHOR_NAME_MAX_ALLOWED_CHAR_COUNT} characters."));
                                    }
                                    name_and_email_raw.get(1).map_or_else(|| lint_failure!(format!("{lint_prefix} is not in its right form. Example of a right form: \"An Author's Name <email@oftheauthor.com>\".")), |maybe_email| {
                                        let maybe_email = maybe_email.split('>').collect::<Vec<_>>();
                                        
                                        // Means that the email field does not finish with `>`.
                                        if maybe_email.len() == 1 {
                                            return lint_failure!(format!("The email field in {lint_prefix} should finish with a '>' character."));
                                        }
                                        maybe_email.get(0).map_or_else(|| lint_failure!(format!(
                                                "The email field in {lint_prefix} can not be parsed."
                                            )), |maybe_email| {
                                            let email_valid = email_address::EmailAddress::is_valid(maybe_email);
                                            if email_valid {
                                                lint_success!(format!("{lint_prefix} exists and is valid."))
                                            } else {
                                                lint_failure!(format!(
                                                    "The email field in {lint_prefix} is invalid."
                                                ))
                                            }
                                        })
                                    })
                                } else {
                                    lint_failure!(format!("The name and email fields in {lint_prefix} is not separated with a white space character."))
                                })
                        },
                    )
                }).collect::<Vec<LintResult>>();
                for result in  lint_results {
                    if let LintResult::Failure(messages) = result {
                       return lint_failure!(format!("Some {lint_prefix} annotations are invalid. More info: {}", messages.join(", ")));
                    }
                }
                lint_success!(format!("All {lint_prefix} annotations are valid."))
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
