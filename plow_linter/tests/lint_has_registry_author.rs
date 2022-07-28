use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::HasRegistryAuthor;

const REGISTRY_AUTHOR_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

// Checks that registry:author field's correct format is 50 chars long and email is validated.
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

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_author_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_author_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_author_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_author_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_author_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_registry_author_f).unwrap();
    let document_g = TurtleDocument::parse_full(&ttl_document_with_registry_author_g).unwrap();
    let document_i = TurtleDocument::parse_full(&ttl_document_with_registry_author_i).unwrap();
    let document_j = TurtleDocument::parse_full(&ttl_document_with_registry_author_j).unwrap();
    let document_k = TurtleDocument::parse_full(&ttl_document_with_registry_author_k).unwrap();
    let document_l = TurtleDocument::parse_full(&ttl_document_with_registry_author_l).unwrap();
    let document_m = TurtleDocument::parse_full(&ttl_document_with_registry_author_m).unwrap();
    let document_n = TurtleDocument::parse_full(&ttl_document_with_registry_author_n).unwrap();
    let document_o = TurtleDocument::parse_full(&ttl_document_with_registry_author_o).unwrap();
    let document_p = TurtleDocument::parse_full(&ttl_document_with_registry_author_p).unwrap();
    let document_r = TurtleDocument::parse_full(&ttl_document_with_registry_author_r).unwrap();

    let lint = HasRegistryAuthor::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);
    let result_g = lint.lint(&document_g);
    let result_i = lint.lint(&document_i);
    let result_j = lint.lint(&document_j);
    let result_k = lint.lint(&document_k);
    let result_l = lint.lint(&document_l);
    let result_m = lint.lint(&document_m);
    let result_n = lint.lint(&document_n);
    let result_o = lint.lint(&document_o);
    let result_p = lint.lint(&document_p);
    let result_r = lint.lint(&document_r);

    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_success());
    assert!(result_d.is_failure());
    assert!(result_e.is_failure());
    assert!(result_f.is_failure());
    assert!(result_g.is_success());
    assert!(result_i.is_failure());
    assert!(result_j.is_failure());
    assert!(result_k.is_failure());
    assert!(result_l.is_failure());
    assert!(result_m.is_success());
    assert!(result_n.is_failure());
    assert!(result_o.is_failure());
    assert!(result_p.is_success());
    assert!(result_r.is_failure());
}

#[test]
fn lint_registry_author_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(REGISTRY_AUTHOR_BASE).is_err());
    assert!(
        TurtleDocument::parse_full(&format!("{REGISTRY_AUTHOR_BASE} registry:author \"\" ."))
            .is_err()
    );
}
