use harriet::TurtleDocument;

use plow_linter::lints::{
    ExistsRegistryLicense, ExistsRegistryLicenseSPDX, HasAtLeastOneValidLicenseAnnotation,
    HasRegistryLicense, HasRegistryLicenseSPDX, PlowLint,
};
use plow_linter::Linter;

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

    let mut linter_a = Linter::try_from(ttl_document_with_registry_license_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint],
        Some(vec![
            Box::new(ExistsRegistryLicense::default()) as PlowLint,
            Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
            Box::new(HasRegistryLicense::default()) as PlowLint,
            Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
        ]),
    );

    let mut linter_b = Linter::try_from(ttl_document_with_registry_license_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint],
        Some(vec![
            Box::new(ExistsRegistryLicense::default()) as PlowLint,
            Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
            Box::new(HasRegistryLicense::default()) as PlowLint,
            Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
        ]),
    );

    let mut linter_c = Linter::try_from(ttl_document_with_registry_license_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint],
        Some(vec![
            Box::new(ExistsRegistryLicense::default()) as PlowLint,
            Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
            Box::new(HasRegistryLicense::default()) as PlowLint,
            Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
        ]),
    );

    let mut linter_d = Linter::try_from(ttl_document_with_registry_license_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint],
        Some(vec![
            Box::new(ExistsRegistryLicense::default()) as PlowLint,
            Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
            Box::new(HasRegistryLicense::default()) as PlowLint,
            Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
        ]),
    );

    let mut linter_e = Linter::try_from(ttl_document_with_registry_license_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint],
        Some(vec![
            Box::new(ExistsRegistryLicense::default()) as PlowLint,
            Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
            Box::new(HasRegistryLicense::default()) as PlowLint,
            Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
        ]),
    );

    let mut linter_f = Linter::try_from(ttl_document_with_registry_license_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint],
        Some(vec![
            Box::new(ExistsRegistryLicense::default()) as PlowLint,
            Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
            Box::new(HasRegistryLicense::default()) as PlowLint,
            Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
        ]),
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_success());
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
    assert!(result_f.first().unwrap().is_failure());
}

#[test]
fn lint_registry_licenses_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_LICENSES_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSES_BASE} registry:license \"\" ."
    ))
    .is_ok());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"\" ."
    ))
    .is_ok());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSES_BASE} registry:licenseSPDX \"\"; registry:license \"\" ."
    ))
    .is_ok());
}
