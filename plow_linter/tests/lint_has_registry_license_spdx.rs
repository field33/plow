use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryLicenseSPDX;

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

    let document_a =
        TurtleDocument::parse_full(&ttl_document_with_registry_license_spdx_a).unwrap();
    let document_b =
        TurtleDocument::parse_full(&ttl_document_with_registry_license_spdx_b).unwrap();
    let document_c =
        TurtleDocument::parse_full(&ttl_document_with_registry_license_spdx_c).unwrap();
    let document_d =
        TurtleDocument::parse_full(&ttl_document_with_registry_license_spdx_d).unwrap();
    let document_e =
        TurtleDocument::parse_full(&ttl_document_with_registry_license_spdx_e).unwrap();

    let lint = HasRegistryLicenseSPDX::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);

    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_failure());
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
}

#[test]
fn lint_registry_license_spdx_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_LICENSE_SPDX_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_LICENSE_SPDX_BASE} registry:licenseSPDX \"\" ."
    ))
    .is_err());
}
