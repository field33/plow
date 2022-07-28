use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryCategory;

const REGISTRY_CATEGORY_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

// Checks that registry:category field's correct format is 50 chars long and email is validated.
#[test]
fn lint_registry_category_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_author_a =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Unknown Category\", \"Another Unknown Category\" .");
    let ttl_document_with_registry_author_b =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Enterprise\", \"Meta Model\" .");
    let ttl_document_with_registry_author_c = format!(
        "{REGISTRY_CATEGORY_BASE} registry:category \"Unknown Category\", \"Meta Model\" ."
    );
    // Duplicate annotations are filtered or not handled in parsing level.
    // So this test would be a success.
    let ttl_document_with_registry_author_d =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Metric\", \"Organization\" , \"Organization\" , \"Organization\" , \"Organization\" .");
    let ttl_document_with_registry_author_e =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Metric\", \"Organization\" , \"Innovation\" , \"Process\" , \"Product\" .");
    let ttl_document_with_registry_author_f =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Metric\", \"Organization\" , \"Innovation\" , \"Process\" , \"Product\" , \"FrameWork\" .");
    let ttl_document_with_registry_author_g =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Unknown Category\"@en, \"Another Unknown Category\" .");

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_author_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_author_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_author_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_author_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_author_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_registry_author_f).unwrap();
    let document_g = TurtleDocument::parse_full(&ttl_document_with_registry_author_g).unwrap();

    let lint = HasRegistryCategory::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);
    let result_g = lint.lint(&document_g);

    assert!(result_a.is_failure());
    assert!(result_b.is_success());
    assert!(result_c.is_failure());
    assert!(result_d.is_success());
    assert!(result_e.is_success());
    assert!(result_f.is_failure());
    assert!(result_g.is_failure());
}

#[test]
fn lint_registry_category_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_CATEGORY_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_CATEGORY_BASE} registry:category \"\" ."
    ))
    .is_err());
}
