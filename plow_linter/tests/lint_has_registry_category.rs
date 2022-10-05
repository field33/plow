use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryCategory, PlowLint};
use plow_linter::Linter;

const REGISTRY_CATEGORY_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_category_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_category_a =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Unknown Category\", \"Another Unknown Category\" .");
    let ttl_document_with_registry_category_b =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Enterprise\", \"Meta Model\" .");
    let ttl_document_with_registry_category_c = format!(
        "{REGISTRY_CATEGORY_BASE} registry:category \"Unknown Category\", \"Meta Model\" ."
    );
    // Duplicate annotations are filtered or not handled in parsing level.
    // So this test would be a success.
    let ttl_document_with_registry_category_d =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Metric\", \"Organization\" , \"Organization\" , \"Organization\" , \"Organization\" .");
    let ttl_document_with_registry_category_e =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Metric\", \"Organization\" , \"Innovation\" , \"Process\" , \"Product\" .");
    let ttl_document_with_registry_category_f =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Metric\", \"Organization\" , \"Innovation\" , \"Process\" , \"Product\" , \"FrameWork\" .");
    let ttl_document_with_registry_category_g =
        format!("{REGISTRY_CATEGORY_BASE} registry:category \"Unknown Category\"@en, \"Another Unknown Category\" .");

    let mut linter_a = Linter::try_from(ttl_document_with_registry_category_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_registry_category_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_registry_category_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_registry_category_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_registry_category_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );
    let mut linter_f = Linter::try_from(ttl_document_with_registry_category_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );
    let mut linter_g = Linter::try_from(ttl_document_with_registry_category_g.as_ref()).unwrap();
    linter_g.add_lint_as_set(
        vec![Box::new(HasRegistryCategory::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();
    let result_g = linter_g.run_all_lints();

    assert!(result_a.first().unwrap().is_failure());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_failure());
    assert!(result_d.first().unwrap().is_success());
    assert!(result_e.first().unwrap().is_success());
    assert!(result_f.first().unwrap().is_failure());
    assert!(result_g.first().unwrap().is_failure());
}

#[test]
fn lint_registry_category_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_CATEGORY_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_CATEGORY_BASE} registry:category \"\" ."
    ))
    .is_ok());
}
