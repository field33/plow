use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryShortDescription;

const REGISTRY_SHORT_DESCRIPTION_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

// Checks that registry:shortDescription field's correct format is 50 chars long and email is validated.
#[test]
fn lint_registry_short_description_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_short_description_a =
        format!("{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"A normal length short description.\"@en .");
    let ttl_document_with_registry_short_description_b =
        format!("{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"A short description which is too long and exceeds the character limit. To exceed the character limit with this example I'll just add some sentences and words which do not have the purpose to exchange some information but instead they serve the only purpose of making this text longer. I think it is long enough now.\"@en .");
    let ttl_document_with_registry_short_description_c =
        format!("{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"A normal length short description with inappropriate asshole words fucking shit.\"@en .");
    let ttl_document_with_registry_short_description_d =
        format!("{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"Multiple short descriptions.\"@en, \"Which are not allowed.\"@en .");
    let ttl_document_with_registry_short_description_e =
        format!("{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"A language tag is necessary.\" .");

    let document_a =
        TurtleDocument::parse_full(&ttl_document_with_registry_short_description_a).unwrap();
    let document_b =
        TurtleDocument::parse_full(&ttl_document_with_registry_short_description_b).unwrap();
    let document_c =
        TurtleDocument::parse_full(&ttl_document_with_registry_short_description_c).unwrap();
    let document_d =
        TurtleDocument::parse_full(&ttl_document_with_registry_short_description_d).unwrap();
    let document_e =
        TurtleDocument::parse_full(&ttl_document_with_registry_short_description_e).unwrap();

    let lint = HasRegistryShortDescription::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);

    assert!(result_a.is_success());
    assert!(result_b.is_failure());
    // Profanity filter turned off.
    assert!(result_c.is_success());
    //
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
}

#[test]
fn lint_registry_short_description_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_SHORT_DESCRIPTION_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"\" ."
    ))
    .is_err());
}
