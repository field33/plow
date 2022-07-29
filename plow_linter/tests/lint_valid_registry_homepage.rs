use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::ValidRegistryHomepage;

const REGISTRY_HOMEPAGE_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

// Checks that registry:homepage field's correct format is 50 chars long and email is validated.
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

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_homepage_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_homepage_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_homepage_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_homepage_d).unwrap();

    let lint = ValidRegistryHomepage::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);

    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_failure());
    assert!(result_d.is_failure());
}

#[test]
fn lint_registry_homepage_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_HOMEPAGE_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_HOMEPAGE_BASE} registry:homepage \"\" ."
    ))
    .is_err());
}
