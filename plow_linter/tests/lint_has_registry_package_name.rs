use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryPackageName;

const REGISTRY_PACKAGE_NAME_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
"#
);

#[test]
fn lint_registry_package_name_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_package_name_a =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"@field33/valid\" .");
    let ttl_document_with_package_name_b =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"@field33/also_valid\" .");
    let ttl_document_with_package_name_c = format!(
        "{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"@field33_is_valid/also_valid\" ."
    );
    let ttl_document_with_package_name_d =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"field33/invalid\" .");
    let ttl_document_with_package_name_e =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"invalid\" .");
    let ttl_document_with_package_name_f =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"/invalid\" .");
    let ttl_document_with_package_name_g =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"@field-33/is-invalid\" .");
    let ttl_document_with_package_name_h =
        format!("{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"@field33/THIS_IS_VALID\" .");

    let document_a = TurtleDocument::parse_full(&ttl_document_with_package_name_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_package_name_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_package_name_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_package_name_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_package_name_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_package_name_f).unwrap();
    let document_g = TurtleDocument::parse_full(&ttl_document_with_package_name_g).unwrap();
    let document_h = TurtleDocument::parse_full(&ttl_document_with_package_name_h).unwrap();

    let lint = HasRegistryPackageName::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);
    let result_g = lint.lint(&document_g);
    let result_h = lint.lint(&document_h);

    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_success());
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
    assert!(result_f.is_failure());
    assert!(result_g.is_failure());
    assert!(result_h.is_success());
}

#[test]
fn lint_registry_package_name_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_PACKAGE_NAME_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"\" ."
    ))
    .is_err());
}
