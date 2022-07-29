use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::ValidRdfsLabels;

const RDFS_LABEL_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:packageName "@test/test" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "2.3.4" ;
rdfs:label "This is an invalid label which misses a language tag and it is over 60 characters but it belongs to the manifest part thus we ignore it in this lint. There is another lint which checks for these." .
<http://field33.com/ontologies/@test/test/>
registry:dependency
"@some/dependency =0.1.0" ,
"@another/dependency >=0.2.0" .
"#
);

// Every Class, ObjectProperty, DataProperty, AnnotationProperty should have an rdfs:label annotation
#[test]
fn lint_related_subjects_with_missing_rdfs_labels_are_invalid() {
    let invalid_document_a = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@en .
<http://field33.com/ontologies/@test/test/other-iri>
rdf:type owl:AnnotationProperty ."
    );
    let invalid_document_b = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@en .
<http://field33.com/ontologies/@test/test/other-iri>
rdf:type owl:DatatypeProperty ."
    );
    let invalid_document_c = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@en .
<http://field33.com/ontologies/@test/test/other-iri>
rdf:type owl:Class ."
    );
    let invalid_document_d = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@en .
<http://field33.com/ontologies/@test/test/other-iri>
rdf:type owl:ObjectProperty ."
    );
    let valid_document_e = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@en .
<http://field33.com/ontologies/@test/test/other-iri>
rdf:type owl:NamedIndividual ."
    );

    let invalid_document_a = TurtleDocument::parse_full(&invalid_document_a).unwrap();
    let invalid_document_b = TurtleDocument::parse_full(&invalid_document_b).unwrap();
    let invalid_document_c = TurtleDocument::parse_full(&invalid_document_c).unwrap();
    let invalid_document_d = TurtleDocument::parse_full(&invalid_document_d).unwrap();
    let valid_document_e = TurtleDocument::parse_full(&valid_document_e).unwrap();

    let lint = ValidRdfsLabels::default();
    let result_a = lint.lint(&invalid_document_a);
    let result_b = lint.lint(&invalid_document_b);
    let result_c = lint.lint(&invalid_document_c);
    let result_d = lint.lint(&invalid_document_d);
    let result_e = lint.lint(&valid_document_e);

    assert!(result_a.is_failure());
    assert!(result_b.is_failure());
    assert!(result_c.is_failure());
    assert!(result_d.is_failure());
    assert!(result_e.is_success());
}

// rdfs:label annotations with a string literal should contain @en as a language tag
#[test]
fn lint_related_subjects_with_rdfs_labels_but_missing_any_or_en_language_tags_are_invalid() {
    let invalid_document_a = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@de ."
    );
    let invalid_document_b = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\" ."
    );
    let valid_document_c = format!(
        "{RDFS_LABEL_BASE}
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"SomeIri\"@en ."
    );

    let invalid_document_a = TurtleDocument::parse_full(&invalid_document_a).unwrap();
    let invalid_document_b = TurtleDocument::parse_full(&invalid_document_b).unwrap();
    let valid_document_c = TurtleDocument::parse_full(&valid_document_c).unwrap();

    let lint = ValidRdfsLabels::default();
    let result_a = lint.lint(&invalid_document_a);
    let result_b = lint.lint(&invalid_document_b);
    let result_c = lint.lint(&valid_document_c);

    assert!(result_a.is_failure());
    assert!(result_b.is_failure());
    assert!(result_c.is_success());
}
