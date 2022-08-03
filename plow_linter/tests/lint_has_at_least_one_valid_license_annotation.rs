use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasAtLeastOneValidLicenseAnnotation;

const REGISTRY_LICENSES_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_at_least_one_registry_license_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_license_a =
        format!("{REGISTRY_LICENSES_BASE} registry:license \"Content\" .");
    let ttl_document_with_registry_license_b =
        format!("{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"MIT\" .");
    let ttl_document_with_registry_license_c = format!(
        "{REGISTRY_LICENSES_BASE} registry:license \"Content \"; registry:licenseSPDX \"MIT\" ."
    );
    let ttl_document_with_registry_license_d = format!(
        "{REGISTRY_LICENSES_BASE} registry:license \"Stuff\"@en ; registry:licenseSPDX \"MIT\"@en  ."
    );
    let ttl_document_with_registry_license_e = format!("{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"MIT\"@en ; registry:license \"Another content \"  .");
    let ttl_document_with_registry_license_f =
      format!("{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"MIT\"@en; registry:license \"Another content \"@en  .");

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_license_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_license_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_license_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_license_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_license_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_registry_license_f).unwrap();

    let lint = HasAtLeastOneValidLicenseAnnotation::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);
    dbg!(&result_b);
    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_success());
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
    assert!(result_f.is_failure());
}

#[test]
fn lint_registry_licenses_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_LICENSES_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSES_BASE} registry:license \"\" ."
    ))
    .is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"\" ."
    ))
    .is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"\"; registry:license \"\" ."
    ))
    .is_err());
}
