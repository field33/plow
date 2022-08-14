use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryLicenseSPDX, PlowLint};
use plow_linter::Linter;

const REGISTRY_LICENSE_SPDX_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_license_spdx_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_license_spdx_a =
        format!("{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"MIT\" .");
    let ttl_document_with_registry_license_spdx_b =
        format!("{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"MIT OR Apache-2.0\" .");
    let ttl_document_with_registry_license_spdx_c = format!(
        "{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"MIT\", \"MIT OR Apache-2.0\" ."
    );
    let ttl_document_with_registry_license_spdx_d =
        format!("{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"A Made Up License-3.0\" .");

    let ttl_document_with_registry_license_spdx_e =
        format!("{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"MIT\"@en .");

    let mut linter_a =
        Linter::try_from(ttl_document_with_registry_license_spdx_a.as_ref()).unwrap();
    linter_a.add_lint(Box::new(HasRegistryLicenseSPDX::default()) as PlowLint);
    let mut linter_b =
        Linter::try_from(ttl_document_with_registry_license_spdx_b.as_ref()).unwrap();
    linter_b.add_lint(Box::new(HasRegistryLicenseSPDX::default()) as PlowLint);
    let mut linter_c =
        Linter::try_from(ttl_document_with_registry_license_spdx_c.as_ref()).unwrap();
    linter_c.add_lint(Box::new(HasRegistryLicenseSPDX::default()) as PlowLint);
    let mut linter_d =
        Linter::try_from(ttl_document_with_registry_license_spdx_d.as_ref()).unwrap();
    linter_d.add_lint(Box::new(HasRegistryLicenseSPDX::default()) as PlowLint);
    let mut linter_e =
        Linter::try_from(ttl_document_with_registry_license_spdx_e.as_ref()).unwrap();
    linter_e.add_lint(Box::new(HasRegistryLicenseSPDX::default()) as PlowLint);

    let result_a = linter_a.run_lints();
    let result_b = linter_b.run_lints();
    let result_c = linter_c.run_lints();
    let result_d = linter_d.run_lints();
    let result_e = linter_e.run_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_failure());
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
}

#[test]
fn lint_registry_license_spdx_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_LICENSE_SPDX_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"\" ."
    ))
    .is_err());
}
