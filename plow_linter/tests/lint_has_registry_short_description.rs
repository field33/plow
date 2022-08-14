use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryShortDescription, PlowLint};
use plow_linter::Linter;

const REGISTRY_SHORT_DESCRIPTION_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

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

    let mut linter_a =
        Linter::try_from(ttl_document_with_registry_short_description_a.as_ref()).unwrap();
    linter_a.add_lint(Box::new(HasRegistryShortDescription::default()) as PlowLint);
    let mut linter_b =
        Linter::try_from(ttl_document_with_registry_short_description_b.as_ref()).unwrap();
    linter_b.add_lint(Box::new(HasRegistryShortDescription::default()) as PlowLint);
    let mut linter_c =
        Linter::try_from(ttl_document_with_registry_short_description_c.as_ref()).unwrap();
    linter_c.add_lint(Box::new(HasRegistryShortDescription::default()) as PlowLint);
    let mut linter_d =
        Linter::try_from(ttl_document_with_registry_short_description_d.as_ref()).unwrap();
    linter_d.add_lint(Box::new(HasRegistryShortDescription::default()) as PlowLint);
    let mut linter_e =
        Linter::try_from(ttl_document_with_registry_short_description_e.as_ref()).unwrap();
    linter_e.add_lint(Box::new(HasRegistryShortDescription::default()) as PlowLint);

    let result_a = linter_a.run_lints();
    let result_b = linter_b.run_lints();
    let result_c = linter_c.run_lints();
    let result_d = linter_d.run_lints();
    let result_e = linter_e.run_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_failure());
    // Profanity filter turned off.
    assert!(result_c.first().unwrap().is_success());
    //
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
}

#[test]
fn lint_registry_short_description_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_SHORT_DESCRIPTION_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{REGISTRY_SHORT_DESCRIPTION_BASE} registry:shortDescription \"\" ."
    ))
    .is_err());
}
