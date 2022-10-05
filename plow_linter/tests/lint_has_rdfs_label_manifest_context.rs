use harriet::TurtleDocument;

use plow_linter::lints::{HasRdfsLabelManifestContext, PlowLint};
use plow_linter::Linter;

const RDFS_LABEL_MANIFEST_CONTEXT_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

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
        format!("{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"Multiple titles.\"@en, \"Which are not allowed.\"@en .");
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

    let mut linter_a = Linter::try_from(ttl_document_with_rdfs_label_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasRdfsLabelManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_rdfs_label_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(HasRdfsLabelManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_rdfs_label_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(HasRdfsLabelManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_rdfs_label_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(HasRdfsLabelManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_rdfs_label_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(HasRdfsLabelManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_f = Linter::try_from(ttl_document_with_rdfs_label_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(HasRdfsLabelManifestContext::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_failure());
    // Profanity filter turned off.
    assert!(result_c.first().unwrap().is_success());
    //
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
    assert!(result_f.first().unwrap().is_success());
}

#[test]
fn lint_registry_rdfs_label_manifest_context_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(RDFS_LABEL_MANIFEST_CONTEXT_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{RDFS_LABEL_MANIFEST_CONTEXT_BASE} rdfs:label \"\" ."
    ))
    .is_ok());
}
