use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryPackageName, PlowLint};
use plow_linter::Linter;

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

    let mut linter_a = Linter::try_from(ttl_document_with_package_name_a.as_ref()).unwrap();
    linter_a.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_b = Linter::try_from(ttl_document_with_package_name_b.as_ref()).unwrap();
    linter_b.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_c = Linter::try_from(ttl_document_with_package_name_c.as_ref()).unwrap();
    linter_c.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_d = Linter::try_from(ttl_document_with_package_name_d.as_ref()).unwrap();
    linter_d.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_e = Linter::try_from(ttl_document_with_package_name_e.as_ref()).unwrap();
    linter_e.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_f = Linter::try_from(ttl_document_with_package_name_f.as_ref()).unwrap();
    linter_f.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_g = Linter::try_from(ttl_document_with_package_name_g.as_ref()).unwrap();
    linter_g.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);
    let mut linter_h = Linter::try_from(ttl_document_with_package_name_h.as_ref()).unwrap();
    linter_h.add_lint(Box::new(HasRegistryPackageName::default()) as PlowLint);

    let result_a = linter_a.run_lints();
    let result_b = linter_b.run_lints();
    let result_c = linter_c.run_lints();
    let result_d = linter_d.run_lints();
    let result_e = linter_e.run_lints();
    let result_f = linter_f.run_lints();
    let result_g = linter_g.run_lints();
    let result_h = linter_h.run_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_success());
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
    assert!(result_f.first().unwrap().is_failure());
    assert!(result_g.first().unwrap().is_failure());
    assert!(result_h.first().unwrap().is_success());
}

#[test]
fn lint_registry_package_name_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_PACKAGE_NAME_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_PACKAGE_NAME_BASE} registry:packageName \"\" ."
    ))
    .is_err());
}
