use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryKeyword;

const REGISTRY_KEYWORD_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

// Checks that registry:keyword field's correct format is 50 chars long and email is validated.
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

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_f).unwrap();
    let document_g = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_g).unwrap();
    let document_h = TurtleDocument::parse_full(&ttl_document_with_registry_keyword_h).unwrap();

    let lint = HasRegistryKeyword::default();
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
    assert!(result_e.is_success());
    assert!(result_f.is_failure());
    assert!(result_g.is_failure());
    assert!(result_h.is_failure());
}

#[test]
fn lint_registry_keyword_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_KEYWORD_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_KEYWORD_BASE} registry:keyword \"\" ."
    ))
    .is_err());
}
