use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryLicense;

const REGISTRY_LICENSE_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_license_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_license_a =
        format!("{REGISTRY_LICENSE_BASE} registry:license \"Computer Associates Trusted Open Source License 1.1\" .");
    let ttl_document_with_registry_license_b =
        format!("{REGISTRY_LICENSE_BASE} registry:license \"Creative Commons Attribution Non Commercial Share Alike 2.0 England and Wales\" .");
    let ttl_document_with_registry_license_c =
        format!("{REGISTRY_LICENSE_BASE} registry:license \"Creative Commons Attribution Non Commercial Share Alike 2.0 England and Wales, and if this license description would exceed 100 characters like it is now it would be a failure\" .");
    let ttl_document_with_registry_license_d =
        format!("{REGISTRY_LICENSE_BASE} registry:license \"CERN Open Hardware Licence Version 2 - Strongly Reciprocal Crappy Suck\" .");
    let ttl_document_with_registry_license_e = format!(
        "{REGISTRY_LICENSE_BASE} registry:license \"copyleft-next 0.3.0\", \"Crossword License\" ."
    );
    let ttl_document_with_registry_license_f =
        format!("{REGISTRY_LICENSE_BASE} registry:license \"A Made Up License-3.0\" .");

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_license_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_license_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_license_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_license_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_license_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_registry_license_f).unwrap();

    let lint = HasRegistryLicense::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);

    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_failure());
    assert!(result_d.is_failure());
    assert!(result_e.is_success());
    assert!(result_f.is_success());
}

#[test]
fn lint_registry_license_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_LICENSE_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSE_BASE} registry:license \"\" ."
    ))
    .is_err());
}
