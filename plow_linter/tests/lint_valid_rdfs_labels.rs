use plow_linter::lints::{PlowLint, ValidRdfsLabels};
use plow_linter::Linter;

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

    let mut linter_a = Linter::try_from(invalid_document_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);
    let mut linter_b = Linter::try_from(invalid_document_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);
    let mut linter_c = Linter::try_from(invalid_document_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);
    let mut linter_d = Linter::try_from(invalid_document_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);
    let mut linter_e = Linter::try_from(valid_document_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();

    assert!(result_a.first().unwrap().is_failure());
    assert!(result_b.first().unwrap().is_failure());
    assert!(result_c.first().unwrap().is_failure());
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_success());
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

    let mut linter_a = Linter::try_from(invalid_document_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);
    let mut linter_b = Linter::try_from(invalid_document_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);
    let mut linter_c = Linter::try_from(valid_document_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(vec![Box::new(ValidRdfsLabels::default()) as PlowLint], None);

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();

    assert!(result_a.first().unwrap().is_failure());
    assert!(result_b.first().unwrap().is_failure());
    assert!(result_c.first().unwrap().is_success());
}
