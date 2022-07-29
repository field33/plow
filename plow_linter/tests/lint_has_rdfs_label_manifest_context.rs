use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRdfsLabelManifestContext;

const RDFS_LABEL_MANIFEST_CONTEXT_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

// Checks that rdfs:label field's correct format is 50 chars long and email is validated.
#[test]
fn lint_registry_rdfs_label_manifest_context_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_rdfs_label_a = format!(
        "{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"A normal title for a field.\"@en ."
    );
    let ttl_document_with_rdfs_label_b =
        format!("{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"A title which is too long for a field because it exceeds the character limit which is 60 chars only.\"@en .");
    let ttl_document_with_rdfs_label_c =
        format!("{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"A title with inappropriate words dick suck shit.\"@en .");
    let ttl_document_with_rdfs_label_d =
        format!("{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"Fuck multiple titles.\"@en, \"Which are allowed because it uses a generic annotation on the other hand only the first one will be evaluated as a title and later ones are ignored in this case this test case should FAIL because of the use of inappropriate words.\"@en .");
    let ttl_document_with_rdfs_label_e =
        format!("{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"A language tag is necessary in a title.\" .");

    let ttl_document_with_rdfs_label_f = format!(
        "{RDFS_LABEL_MANIFEST_CONTEXT_BASE}
rdfs:label \"A normal title for a field.\"@en .
<http://field33.com/ontologies/@test/test/some-iri>
rdf:type owl:AnnotationProperty ;
rdfs:label \"These other labels such as this one should be ignored in this lint so we assert that this one is a success.\"@en .
<http://field33.com/ontologies/@test/test/other-iri>
rdf:type owl:NamedIndividual ."
    );

    let document_a = TurtleDocument::parse_full(&ttl_document_with_rdfs_label_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_rdfs_label_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_rdfs_label_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_rdfs_label_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_rdfs_label_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_rdfs_label_f).unwrap();

    let lint = HasRdfsLabelManifestContext::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);

    assert!(result_a.is_success());
    assert!(result_b.is_failure());
    assert!(result_c.is_failure());
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
    assert!(result_f.is_success());
}

#[test]
fn lint_registry_rdfs_label_manifest_context_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(RDFS_LABEL_MANIFEST_CONTEXT_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"\" ."
    ))
    .is_err());
}
