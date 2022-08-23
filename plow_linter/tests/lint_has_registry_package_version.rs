use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryPackageVersion, PlowLint};
use plow_linter::Linter;

const REGISTRY_PACKAGE_VERSION_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:packageName "@test/test" ;
registry:canonicalPrefix "test" ;
"#
);

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
    let mut linter_a = Linter::try_from(ttl_document_with_package_version_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_package_version_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_package_version_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_package_version_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_package_version_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_f = Linter::try_from(ttl_document_with_package_version_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_g = Linter::try_from(ttl_document_with_package_version_g.as_ref()).unwrap();
    linter_g.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_h = Linter::try_from(ttl_document_with_package_version_h.as_ref()).unwrap();
    linter_h.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_i = Linter::try_from(ttl_document_with_package_version_i.as_ref()).unwrap();
    linter_i.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );
    let mut linter_j = Linter::try_from(ttl_document_with_package_version_j.as_ref()).unwrap();
    linter_j.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();
    let result_g = linter_g.run_all_lints();
    let result_h = linter_h.run_all_lints();
    let result_i = linter_i.run_all_lints();
    let result_j = linter_j.run_all_lints();
    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_failure());
    assert!(result_c.first().unwrap().is_failure());
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
    assert!(result_f.first().unwrap().is_failure());
    assert!(result_g.first().unwrap().is_failure());
    assert!(result_h.first().unwrap().is_failure());
    assert!(result_i.first().unwrap().is_failure());
    assert!(result_j.first().unwrap().is_failure());
}

#[test]
fn lint_registry_package_version_exists_and_invalid() {
    let ttl_document_with_package_version =
        format!("{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"invalid_stuff\" .");

    let mut linter_a = Linter::try_from(ttl_document_with_package_version.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasRegistryPackageVersion::default()) as PlowLint],
        None,
    );

    let result = linter_a.run_all_lints();
    assert!(result.first().unwrap().is_failure());
}

#[test]
fn lint_registry_package_version_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_PACKAGE_VERSION_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_PACKAGE_VERSION_BASE} registry:packageVersion \"\" ."
    ))
    .is_err());
}
