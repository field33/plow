use harriet::TurtleDocument;

use plow_linter::lints::{HasRegistryAuthor, PlowLint};
use plow_linter::Linter;

const REGISTRY_AUTHOR_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_author_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_author_a =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"Ali Somay <ali@field33.com>\" .");
    let ttl_document_with_registry_author_b =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"陈 <chen@field33.com>\" .");
    let ttl_document_with_registry_author_c =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"A Probably Long Portuguese Name <ricardo@field33.com>\" .");
    let ttl_document_with_registry_author_d =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"Invalid<Name <test@field33.com>\" .");
    let ttl_document_with_registry_author_e = format!(
        "{REGISTRY_AUTHOR_BASE} registry:author \"Another>Invalid Name <test@field33.com>\" ."
    );
    let ttl_document_with_registry_author_f =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"NoSpace<test@field33.com>\" .");
    let ttl_document_with_registry_author_g =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"   MoreSpace   <test@field33.com>\" .");
    let ttl_document_with_registry_author_i =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"A Name Which Exceeds 50 Characters Which Is Pretty Long And We Don't Want To Support It So We Catch It In Validation <test@field33.com>\" .");

    let ttl_document_with_registry_author_j = format!(
        "{REGISTRY_AUTHOR_BASE} registry:author \"An Email With Wrong Format test@field33.com>\" ."
    );
    let ttl_document_with_registry_author_k = format!(
        "{REGISTRY_AUTHOR_BASE} registry:author \"Another Email With Wrong Format <test@field33.com\" ."
    );
    let ttl_document_with_registry_author_l =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"An Invalid Email <testfield33.com>\" .");
    let ttl_document_with_registry_author_m = format!(
        "{REGISTRY_AUTHOR_BASE} registry:author \"Surprising but a valid email <test@field33>\" ."
    );
    let ttl_document_with_registry_author_n = format!(
        "{REGISTRY_AUTHOR_BASE} registry:author \"Another Email With Wrong Format <test@field33.com陈\" ."
    );
    let ttl_document_with_registry_author_o = format!(
        "{REGISTRY_AUTHOR_BASE} registry:author \"Another Email With Wrong Format <test@field33.com陈\"; registry:author \"Ali Somay <ali@field33.com>\" ."
    );
    let ttl_document_with_registry_author_p =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"Ali Somay <ali@field33.com>\"; registry:author \"Freddy Mercury <fred@field33.com>\" .");
    let ttl_document_with_registry_author_r =
        format!("{REGISTRY_AUTHOR_BASE} registry:author \"Fucking Dick <suck@crap.com>\" .");

    let mut linter_a = Linter::try_from(ttl_document_with_registry_author_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_registry_author_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_registry_author_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_registry_author_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_registry_author_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_f = Linter::try_from(ttl_document_with_registry_author_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_g = Linter::try_from(ttl_document_with_registry_author_g.as_ref()).unwrap();
    linter_g.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_i = Linter::try_from(ttl_document_with_registry_author_i.as_ref()).unwrap();
    linter_i.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_j = Linter::try_from(ttl_document_with_registry_author_j.as_ref()).unwrap();
    linter_j.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_k = Linter::try_from(ttl_document_with_registry_author_k.as_ref()).unwrap();
    linter_k.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_l = Linter::try_from(ttl_document_with_registry_author_l.as_ref()).unwrap();
    linter_l.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_m = Linter::try_from(ttl_document_with_registry_author_m.as_ref()).unwrap();
    linter_m.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_n = Linter::try_from(ttl_document_with_registry_author_n.as_ref()).unwrap();
    linter_n.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_o = Linter::try_from(ttl_document_with_registry_author_o.as_ref()).unwrap();
    linter_o.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_p = Linter::try_from(ttl_document_with_registry_author_p.as_ref()).unwrap();
    linter_p.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );
    let mut linter_r = Linter::try_from(ttl_document_with_registry_author_r.as_ref()).unwrap();
    linter_r.add_lint_as_set(
        vec![Box::new(HasRegistryAuthor::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();
    let result_g = linter_g.run_all_lints();
    let result_i = linter_i.run_all_lints();
    let result_j = linter_j.run_all_lints();
    let result_k = linter_k.run_all_lints();
    let result_l = linter_l.run_all_lints();
    let result_m = linter_m.run_all_lints();
    let result_n = linter_n.run_all_lints();
    let result_o = linter_o.run_all_lints();
    let result_p = linter_p.run_all_lints();
    let result_r = linter_r.run_all_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_success());
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
    assert!(result_f.first().unwrap().is_failure());
    assert!(result_g.first().unwrap().is_success());
    assert!(result_i.first().unwrap().is_failure());
    assert!(result_j.first().unwrap().is_failure());
    assert!(result_k.first().unwrap().is_failure());
    assert!(result_l.first().unwrap().is_failure());
    assert!(result_m.first().unwrap().is_success());
    assert!(result_n.first().unwrap().is_failure());
    assert!(result_o.first().unwrap().is_failure());
    assert!(result_p.first().unwrap().is_success());
    assert!(result_r.first().unwrap().is_failure());
}

#[test]
fn lint_registry_author_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_AUTHOR_BASE).is_err());
    assert!(
        TurtleDocument::parse_full(&format!("{REGISTRY_AUTHOR_BASE} registry:author \"\" ."))
            .is_ok()
    );
}
