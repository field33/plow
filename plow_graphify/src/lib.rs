#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,

    // We decided that we're ok with expect
    clippy::expect_used,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,
    // Currently breaks CI, let's wait a bit more until new clippy version is more spread.
    // clippy::single_char_lifetime_names,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    clippy::wildcard_dependencies,
    clippy::redundant_feature_names,
    clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,

    clippy::negative_feature_names
)]

use field33_rdftk_core_temporary_fork::model::graph::GraphRef;
use field33_rdftk_core_temporary_fork::model::literal::LanguageTag;
use field33_rdftk_core_temporary_fork::model::statement::{ObjectNodeRef, StatementList};
use field33_rdftk_core_temporary_fork::simple;
use field33_rdftk_core_temporary_fork::simple::indexed::graph_factory;
use field33_rdftk_iri_temporary_fork::IRI;
use harriet::{
    Directive, IRIReference, Literal, Object, PrefixedName, Statement as HarrietStatement, Subject,
    Triples, TurtleDocument, Verb, IRI as HarrietIRI,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RDFParseError {
    #[error("Unresolvable prefix: {0}")]
    UnresolvablePrefix(String),
    #[error("Unsupported structure")]
    UnsupportedStructure,
}

// TODO: This function can be improved in the future.
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::too_many_lines)]
pub fn document_to_graph(document: &TurtleDocument) -> Result<GraphRef, RDFParseError> {
    let mut statements: StatementList = vec![];
    let factory = simple::statement::statement_factory();
    let literal_factory = simple::literal::literal_factory();

    let mut prefix_map: HashMap<Option<String>, IRIReference> = HashMap::new();

    for statement in &document.statements {
        match statement {
            HarrietStatement::Directive(directive) => {
                // TODO: Ignoring @base, etc. for now
                if let Directive::Prefix(prefix) = directive {
                    prefix_map.insert(
                        prefix.prefix.as_ref().map(ToString::to_string),
                        prefix.iri.clone(),
                    );
                }
            }
            HarrietStatement::Triples(triples) => match triples {
                Triples::Labeled(_, subject, predicate_object_list) => match subject {
                    Subject::IRI(subject) => {
                        let subject = resolve_iri(&prefix_map, subject)?;
                        for (_, verb, object_list, _) in &predicate_object_list.list {
                            match verb {
                                Verb::IRI(predicate) => {
                                    let predicate = resolve_iri(&prefix_map, predicate)?;
                                    for (_, _, object) in &object_list.list {
                                        match object {
                                            Object::IRI(object) => {
                                                let object = resolve_iri(&prefix_map, object)?;
                                                // We know the input is valid because `harriet_to_rdftk_iri` only accepts valid input.
                                                #[allow(clippy::unwrap_used)]
                                                statements.push(
                                                    factory
                                                        .statement(
                                                            factory.named_subject(
                                                                harriet_to_rdftk_iri(&subject)
                                                                    .into(),
                                                            ),
                                                            harriet_to_rdftk_iri(&predicate).into(),
                                                            factory.named_object(
                                                                harriet_to_rdftk_iri(&object)
                                                                    .into(),
                                                            ),
                                                        )
                                                        .unwrap(),
                                                );
                                            }
                                            Object::Literal(literal) => match literal {
                                                Literal::RDFLiteral(rdf_literal) => {
                                                    let string = rdf_literal.string.to_string();
                                                    let object_node: ObjectNodeRef;
                                                    if let Some(ref language_tag) =
                                                        rdf_literal.language_tag
                                                    {
                                                        // Does not check or report errors on malformed language tags
                                                        // Instead it would just ignore it.
                                                        if let Ok(tag) =
                                                            LanguageTag::from_str(language_tag)
                                                        {
                                                            object_node = factory.literal_object(
                                                                literal_factory
                                                                    .with_language(&string, tag),
                                                            );
                                                        } else {
                                                            object_node = factory.literal_object(
                                                                literal_factory.string(&string),
                                                            );
                                                        }
                                                    } else {
                                                        object_node = factory.literal_object(
                                                            literal_factory.string(&string),
                                                        );
                                                    }
                                                    // We know the input is valid because `harriet_to_rdftk_iri` only accepts valid input.
                                                    // Also `object_node` variable will always be a valid `ObjectNodeRef`.
                                                    #[allow(clippy::unwrap_used)]
                                                    statements.push(
                                                        factory
                                                            .statement(
                                                                factory.named_subject(
                                                                    harriet_to_rdftk_iri(&subject)
                                                                        .into(),
                                                                ),
                                                                harriet_to_rdftk_iri(&predicate)
                                                                    .into(),
                                                                object_node,
                                                            )
                                                            .unwrap(),
                                                    );
                                                }
                                                Literal::BooleanLiteral(_)
                                                | Literal::NumericLiteral(_) => {
                                                    return Err(
                                                        RDFParseError::UnsupportedStructure,
                                                    );
                                                }
                                            },
                                            _ => {
                                                // TODO
                                                // return Err(RDFParseError::UnsupportedStructure);
                                            }
                                        }
                                    }
                                }
                                Verb::A => {
                                    return Err(RDFParseError::UnsupportedStructure);
                                }
                            }
                        }
                    }
                    Subject::Collection(_) | Subject::BlankNode(_) => {
                        // TODO
                        return Err(RDFParseError::UnsupportedStructure);
                    }
                },
            },
        }
    }

    let graph_factory = graph_factory();
    let graph = graph_factory.graph_from(&statements, None);

    Ok(graph)
}

fn harriet_to_rdftk_iri(iri: &IRIReference) -> IRI {
    // We know that our input is a valid IRI, so we can unwrap
    #[allow(clippy::unwrap_used)]
    IRI::from_str(&iri.iri).unwrap()
}

fn resolve_iri<'iri>(
    prefix_map: &HashMap<Option<String>, IRIReference>,
    iri: &'iri HarrietIRI<'iri>,
) -> Result<IRIReference<'iri>, RDFParseError> {
    match iri {
        HarrietIRI::IRIReference(iri_ref) => Ok(iri_ref.clone()),
        HarrietIRI::PrefixedName(prefixed_name) => resolve_prefixed_name(prefix_map, prefixed_name),
    }
}

fn resolve_prefixed_name<'iri>(
    prefix_map: &HashMap<Option<String>, IRIReference>,
    prefixed_name: &PrefixedName<'iri>,
) -> Result<IRIReference<'iri>, RDFParseError> {
    let prefix_mapping = prefix_map.get(&prefixed_name.prefix.as_ref().map(ToString::to_string));
    prefix_mapping
        .map(|prefix| {
            let joined_iri = format!(
                "{}{}",
                prefix.iri,
                prefixed_name.name.as_ref().unwrap_or(&Cow::Borrowed(""))
            );
            IRIReference {
                iri: Cow::Owned(joined_iri),
            }
        })
        .ok_or_else(|| {
            RDFParseError::UnresolvablePrefix(
                prefixed_name
                    .name
                    .as_ref()
                    .unwrap_or(&Cow::Borrowed(""))
                    .to_string(),
            )
        })
}

// #[cfg(test)]
// mod tests {
//     // We don't explicitly need restrictive lints for tests.
//     #![allow(clippy::restriction)]

//     use super::*;
//     use harriet::TurtleDocument;

//     #[test]
//     fn ontology_declaration() {
//         let ontology = r#"
//         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
//         @prefix owl: <http://www.w3.org/2002/07/owl#> .

//         <http://field33.com/ontologies/EXAMPLE_ONTOLOGY/> rdf:type owl:Ontology .
//             "#;
//         let document = TurtleDocument::parse_full(ontology).unwrap();
//         let graph = document_to_graph(&document).unwrap();
//         let graph_borrow = graph.borrow();

//         let factory = simple::statement::statement_factory();

//         assert_eq!(
//             graph_borrow
//                 .matches(
//                     Some(
//                         &factory.named_subject(
//                             IRI::from_str("http://field33.com/ontologies/EXAMPLE_ONTOLOGY/")
//                                 .unwrap()
//                                 .into()
//                         )
//                     ),
//                     Some(
//                         &IRI::from_str("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
//                             .unwrap()
//                             .into()
//                     ),
//                     Some(
//                         &factory.named_object(
//                             IRI::from_str("http://www.w3.org/2002/07/owl#Ontology")
//                                 .unwrap()
//                                 .into()
//                         )
//                     ),
//                 )
//                 .len(),
//             1
//         );
//     }
// }
