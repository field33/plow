use crate::lint::{
    common_error_literals::{NO_ROOT_PREFIX, RDF_GRAPH_PARSE_ERROR},
    helpers::catch_single_or_multiple_annotations_which_must_exist,
    lint_failure, lint_success, Lint, LintResult,
};
use harriet::TurtleDocument;
use plow_graphify::document_to_graph;
use plow_ontology::constants::REGISTRY_CATEGORIES;
use plow_package_management::metadata::get_root_prefix;
use rdftk_iri::IRI as RDFTK_IRI;
use std::collections::HashSet;
use std::str::FromStr;

const RELATED_FIELD: &str = "`registry:category`";
const MAX_CATEGORIES: usize = 5;
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
    fn short_description(&self) -> &str {
        "Check that the ontology is annotated with a value for `registry:category`"
    }
    /// Lints for the existence of `registry:category` and its correct format
    /// (should be `@namespace/package_name` , with both the namespace and package name
    /// only being alphanumeric characters + underscore)
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
                                == &RDFTK_IRI::from_str(REGISTRY_CATEGORIES).unwrap().into()
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

                                let category_raw = literal.lexical_form().trim();
                                if checked_literals.contains(&category_raw) {
                                   return lint_failure!(format!(
                                    "Each {RELATED_FIELD} annotation can only be used only once. There could be no duplicates."
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
                lint_success!("All {RELATED_FIELD} annotations are valid.")
            } else {
                lint_failure!(NO_ROOT_PREFIX)
            }
        } else {
            lint_failure!(RDF_GRAPH_PARSE_ERROR)
        }
    }
}
