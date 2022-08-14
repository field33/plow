use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryKeyword, PlowLint};
use plow_linter::Linter;

const REGISTRY_KEYWORD_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_keyword_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_keyword_a =
        format!("{REGISTRY_KEYWORD_BASE} registry:keyword \"A keyword\" .");
    let ttl_document_with_registry_keyword_b =
        format!("{REGISTRY_KEYWORD_BASE} registry:keyword \"Multiple\", \"Keywords\" .");
    let ttl_document_with_registry_keyword_c =
        format!("{REGISTRY_KEYWORD_BASE} registry:keyword \"Exactly\", \"Maximum\", \"Allowed\", \"Number Of\", \"Keywords\" .");
    let ttl_document_with_registry_keyword_d =
        format!("{REGISTRY_KEYWORD_BASE} registry:keyword \"More\", \"Than\" , \"Allowed\", \"Number\" , \"Of\", \"Keywords\" .");
    // Duplicate annotations are filtered or not handled in parsing level.
    // So this test would be a success.
    let ttl_document_with_registry_keyword_e =
        format!("{REGISTRY_KEYWORD_BASE} registry:keyword \"Duplicate\", \"Duplicate\" , \"Keywords\" , \"Keywords\" , \"Keywords\" .");
    let ttl_document_with_registry_keyword_f =
        format!("{REGISTRY_KEYWORD_BASE} registry:keyword \"Some\", \"Fucking\" , \"Shit\" , \"Keywords\" .");
    let ttl_document_with_registry_keyword_g = format!(
        "{REGISTRY_KEYWORD_BASE} registry:keyword \"Language\"@en, \"Tag Containing Keywords\" ."
    );
    let ttl_document_with_registry_keyword_h = format!(
        "{REGISTRY_KEYWORD_BASE} registry:keyword \"An absurdly long keyword which exceeds the character limit and should not be allowed because it is too much overhead on indexing and does not make sense to call it a keyword in the first place\", \"Tag Containing Keywords\" ."
    );

    let mut linter_a = Linter::try_from(ttl_document_with_registry_keyword_a.as_ref()).unwrap();
    linter_a.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_b = Linter::try_from(ttl_document_with_registry_keyword_b.as_ref()).unwrap();
    linter_b.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_c = Linter::try_from(ttl_document_with_registry_keyword_c.as_ref()).unwrap();
    linter_c.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_d = Linter::try_from(ttl_document_with_registry_keyword_d.as_ref()).unwrap();
    linter_d.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_e = Linter::try_from(ttl_document_with_registry_keyword_e.as_ref()).unwrap();
    linter_e.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_f = Linter::try_from(ttl_document_with_registry_keyword_f.as_ref()).unwrap();
    linter_f.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_g = Linter::try_from(ttl_document_with_registry_keyword_g.as_ref()).unwrap();
    linter_g.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);
    let mut linter_h = Linter::try_from(ttl_document_with_registry_keyword_h.as_ref()).unwrap();
    linter_h.add_lint(Box::new(HasRegistryKeyword::default()) as PlowLint);

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
    assert!(result_e.first().unwrap().is_success());
    assert!(result_f.first().unwrap().is_failure());
    assert!(result_g.first().unwrap().is_failure());
    assert!(result_h.first().unwrap().is_failure());
}

#[test]
fn lint_registry_keyword_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_KEYWORD_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_KEYWORD_BASE} registry:keyword \"\" ."
    ))
    .is_err());
}
