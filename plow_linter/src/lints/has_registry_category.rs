use crate::{lint::{
    common_error_literals::{NO_ROOT_PREFIX, },
    helpers::{catch_single_or_multiple_annotations_which_must_exist, fail_if_has_language_tag},
    lint_failure, lint_success, Lint, LintResult,
}, Linter, MultiReaderRdfGraph};


use plow_ontology::constants::REGISTRY_CATEGORY;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::{collections::HashSet, any::Any};
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:category`";
/// Maximum allowed categories.
const MAX_CATEGORIES: usize = 5;
/// Available categories to choose from.
const CATEGORY_ALLOW_LIST: [&str; 30] = [
    "Benchmark",
    "Design",
    "Enterprise",
    "Framework",
    "Innovation",
    "Meta Model",
    "Methodology",
    "Metric",
    "Organization",
    "People",
    "Portfolio",
    "Process",
    "Product",
    "Project Management",
    "Software Architecture",
    "Software Development",
    "Software Implementation",
    "Software Infrastructure",
    "Strategy",
    "Transformation",
    "Upper Ontology",
    "Customer Ontology",
    "Scrum",
    "Relation",
    "Communication",
    "Core",
    "Graph Visualization",
    "Change Tracking",
    "Graph Style",
    "Interoperability",
];
/// Ensures that a value for `registry:category` is specified as annotation on the ontology.
#[derive(Debug, Default)]
pub struct HasRegistryCategory;

impl Lint for HasRegistryCategory {
    fn as_any(&self) -> &dyn Any {
        self
    }     
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:category`"
    }
    /// Lints for the existence of `registry:category` and its validity.
    /// Available categories are defined by plow. 
    /// Maximum 5 categories are allowed.
    /// Categories shouldn't contain language tags.
    /// 
    /// Here is a list: 
    /// ```rust
    /// const CATEGORY_ALLOW_LIST: [&str; 30] = [
    /// "Benchmark",
    /// "Design",
    /// "Enterprise",
    /// "Framework",
    /// "Innovation",
    /// "Meta Model",
    /// "Methodology",
    /// "Metric",
    /// "Organization",
    /// "People",
    /// "Portfolio",
    /// "Process",
    /// "Product",
    /// "Project Management",
    /// "Software Architecture",
    /// "Software Development",
    /// "Software Implementation",
    /// "Software Infrastructure",
    /// "Strategy",
    /// "Transformation",
    /// "Upper Ontology",
    /// "Customer Ontology",
    /// "Scrum",
    /// "Relation",
    /// "Communication",
    /// "Core",
    /// "Graph Visualization",
    /// "Change Tracking",
    /// "Graph Style",
    /// "Interoperability",
    /// ];
    /// ``` 
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
                                == &RDFTK_IRI::from_str(REGISTRY_CATEGORY).unwrap().into()
                    })
                    .collect::<HashSet<_>>();

                if let Some(failure) = catch_single_or_multiple_annotations_which_must_exist(
                    &annotations,
                    RELATED_FIELD,
                ) {
                    return failure;
                }

                let lint_prefix = format!("The value of {RELATED_FIELD},");
                if annotations.len() > MAX_CATEGORIES {
                    return lint_failure!(format!(
                        "{lint_prefix} can not contain more than {MAX_CATEGORIES} categories. Please reduce the amount of categories.",
                    ));
                }

                // Checking for duplicate categories are actually unnecessary. Because they're checked on parsing level.
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
                                let category_raw = literal.lexical_form().trim();
                                if checked_literals.contains(&category_raw) {
                                   return lint_failure!(format!(
                                    "Each {RELATED_FIELD} value can only be used only once. There could be no duplicates."
                                   ));
                                }
                                checked_literals.push(category_raw);
                                
                                if CATEGORY_ALLOW_LIST.contains(&category_raw) {                         
                                    return lint_success!(format!("{lint_prefix} is valid."));                              
                                }
                                
                                lint_failure!(format!(
                                    "{lint_prefix} does contain a category ({category_raw}) which is not available."
                                ))
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
