use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryPackageVersion;

const REGISTRY_PACKAGE_VERSION_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:packageName "@test/test" ;
registry:canonicalPrefix "test" ;
"#
);

// Only simple and fully complete version strings are allowed with no prefix or suffixes. e.g. major.minor.patch.
#[test]
fn lint_registry_package_version_exists_and_valid() {
    let ttl_document_with_package_version_a =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"2.3.4\" .");
    let ttl_document_with_package_version_b =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \">=1.2.3\" .");
    let ttl_document_with_package_version_c =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"2.x\" .");
    let ttl_document_with_package_version_d =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \">=1.0.0, <=2.3.4\" .");
    let ttl_document_with_package_version_e =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"*\" .");
    let ttl_document_with_package_version_f =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"=1.0.0\" .");
    let ttl_document_with_package_version_g =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"1.0\" .");
    let ttl_document_with_package_version_h =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"^1.0.0\" .");
    let ttl_document_with_package_version_i =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"=1.0.0-beta.1\" .");
    let ttl_document_with_package_version_j =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"=1.0.0-alpha.1+001\" .");
    let document_a = TurtleDocument::parse_full(&ttl_document_with_package_version_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_package_version_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_package_version_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_package_version_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_package_version_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_package_version_f).unwrap();
    let document_g = TurtleDocument::parse_full(&ttl_document_with_package_version_g).unwrap();
    let document_h = TurtleDocument::parse_full(&ttl_document_with_package_version_h).unwrap();
    let document_i = TurtleDocument::parse_full(&ttl_document_with_package_version_i).unwrap();
    let document_j = TurtleDocument::parse_full(&ttl_document_with_package_version_j).unwrap();
    let lint = HasRegistryPackageVersion::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);
    let result_g = lint.lint(&document_g);
    let result_h = lint.lint(&document_h);
    let result_i = lint.lint(&document_i);
    let result_j = lint.lint(&document_j);
    assert!(result_a.is_success());
    assert!(result_b.is_failure());
    assert!(result_c.is_failure());
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
    assert!(result_f.is_failure());
    assert!(result_g.is_failure());
    assert!(result_h.is_failure());
    assert!(result_i.is_failure());
    assert!(result_j.is_failure());
}

#[test]
fn lint_registry_package_version_exists_and_invalid() {
    let ttl_document_with_package_version =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"invalid_stuff\" .");
    let document = TurtleDocument::parse_full(&ttl_document_with_package_version).unwrap();
    let lint = HasRegistryPackageVersion::default();
    let result = lint.lint(&document);
    assert!(result.is_failure());
}

#[test]
fn lint_registry_package_version_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_PACKAGE_VERSION_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"\" ."
    ))
    .is_err());
}
