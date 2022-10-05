use harriet::TurtleDocument;

use plow_linter::lints::{PlowLint, ValidRegistryHomepage};
use plow_linter::Linter;

const REGISTRY_HOMEPAGE_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_homepage_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_homepage_a =
        format!("{REGISTRY_HOMEPAGE_BASE} registry:homepage \"beloved_field.com\" .");
    let ttl_document_with_registry_homepage_b =
        format!("{REGISTRY_HOMEPAGE_BASE} registry:homepage \"https://www.beloved-field.io\" .");
    let ttl_document_with_registry_homepage_c =
        format!("{REGISTRY_HOMEPAGE_BASE} registry:homepage \"https://www.beloved-field.io\", \"http://www.another-beloved-field.io\" .");
    let ttl_document_with_registry_homepage_d =
        format!("{REGISTRY_HOMEPAGE_BASE} registry:homepage \"https://_i-am-a-very-long-long-string-of-text-that-should-not-be-allowed.example.com\" .");

    let mut linter_a = Linter::try_from(ttl_document_with_registry_homepage_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(ValidRegistryHomepage::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_registry_homepage_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(ValidRegistryHomepage::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_registry_homepage_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(ValidRegistryHomepage::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_registry_homepage_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(ValidRegistryHomepage::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_failure());
    assert!(result_d.first().unwrap().is_success());
}

#[test]
fn lint_registry_homepage_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_HOMEPAGE_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_HOMEPAGE_BASE} registry:homepage \"\" ."
    ))
    .is_ok());
}
