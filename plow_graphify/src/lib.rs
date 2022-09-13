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
use field33_rdftk_core_temporary_fork::model::literal::{LiteralFactoryRef, LiteralRef};
use field33_rdftk_core_temporary_fork::model::statement::StatementList;
use field33_rdftk_core_temporary_fork::simple;
use field33_rdftk_core_temporary_fork::simple::indexed::graph_factory;
use field33_rdftk_iri_temporary_fork::IRI;
use harriet::triple_production::{
    RdfIri, RdfLiteral, RdfObject, RdfPredicate, RdfSubject, TripleProducer,
};
use harriet::TurtleDocument;
use std::str::FromStr;
use std::sync::Arc;
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

    let triples = TripleProducer::produce_for_document(&document).unwrap();

    // TODO: implement blank nodes
    // let blank_node_counter = 0;
    // let blank_nodes_map = HashMap::<RdfBlankNode, String>::default();

    for triple in triples {
        let subject = match triple.subject {
            RdfSubject::IRI(iri) => factory.named_subject(rdf_iri_to_rdftk_iri(iri).into()),
            RdfSubject::BlankNode(_) => {
                // TODO: implement blank nodes
                continue;
            }
        };
        let predciate = match triple.predicate {
            RdfPredicate::IRI(iri) => rdf_iri_to_rdftk_iri(iri),
        };
        let object = match triple.object {
            RdfObject::IRI(iri) => factory.named_object(rdf_iri_to_rdftk_iri(iri).into()),
            RdfObject::BlankNode(_) => {
                // TODO: implement blank nodes
                continue;
            }
            RdfObject::Literal(literal) => factory.literal_object(rdf_literal_to_rdftk_literal(
                literal_factory.clone(),
                literal,
            )?),
        };

        statements.push(
            factory
                .statement(subject, predciate.into(), object)
                .unwrap(),
        )
    }

    let graph_factory = graph_factory();
    let graph = graph_factory.graph_from(&statements, None);

    Ok(graph)
}

fn rdf_iri_to_rdftk_iri(iri: RdfIri) -> IRI {
    IRI::from_str(iri.iri.as_ref()).unwrap()
}

fn rdf_literal_to_rdftk_literal(
    literal_factory: LiteralFactoryRef,
    literal: RdfLiteral,
) -> Result<LiteralRef, RDFParseError> {
    Ok(
        match (
            literal.lexical_form,
            literal.datatype_iri,
            literal.language_tag,
        ) {
            (lexical_form, _, Some(language_tag)) => literal_factory
                .with_language_str(lexical_form.as_ref(), language_tag.as_ref())
                .map_err(|_| RDFParseError::UnsupportedStructure)?,
            (lexical_form, Some(data_type), _) => literal_factory.with_data_type(
                lexical_form.as_ref(),
                Arc::new(rdf_iri_to_rdftk_iri(data_type)).into(),
            ),
            (lexical_form, None, None) => literal_factory.literal(lexical_form.as_ref()),
        },
    )
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
