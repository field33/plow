use harriet::TurtleDocument;
use plow_linter::lint::Lint;
use plow_linter::lints::ValidRegistryDependencies;

const REGISTRY_DEPENDENCY_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:packageName "@test/test" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "2.3.4" .
<http://field33.com/ontologies/@test/test/>
"#
);

// Check for validity of version requirement specification in registry:dependency annotation
// (format to be defined, but very likely same as permitted in Cargo) currently all semver forms are allowed.
#[test]
fn lint_registry_package_dependencies_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_registry_dependency_a =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency =0.1.0\" .");
    let ttl_document_with_registry_dependency_b =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency =0.1.0\", \"@another/dependency >=0.2.0\" .");
    let ttl_document_with_registry_dependency_c =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"some/dependency =0.1.0\", \"@another/dependency =0.2.0\" .");
    let ttl_document_with_registry_dependency_d =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency >=1.2.3, <1.8.0\", \"@another/dependency ^3\" .");
    let ttl_document_with_registry_dependency_e =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency 0.1.0\" .");

    let document_a = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_a).unwrap();
    let document_b = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_b).unwrap();
    let document_c = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_c).unwrap();
    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_e).unwrap();

    let lint = ValidRegistryDependencies::default();
    let result_a = lint.lint(&document_a);
    let result_b = lint.lint(&document_b);
    let result_c = lint.lint(&document_c);
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);

    assert!(result_a.is_success());
    assert!(result_b.is_success());
    assert!(result_c.is_failure());
    assert!(result_d.is_success());
    assert!(result_e.is_failure());
}

#[test]
fn lint_registry_package_dependencies_with_wildcards_emit_warning() {
    let ttl_document_with_registry_dependency_d =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency ^5.1.0\", \"@another/dependency *\" .");
    let ttl_document_with_registry_dependency_e =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency ~0.1.0\", \"@another/dependency 0.2.*\" .");
    let ttl_document_with_registry_dependency_f =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency ~0.1.x\" .");
    let ttl_document_with_registry_dependency_g =
        format!("{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency 0.1.x\" .");

    let document_d = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_d).unwrap();
    let document_e = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_e).unwrap();
    let document_f = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_f).unwrap();
    let document_g = TurtleDocument::parse_full(&ttl_document_with_registry_dependency_g).unwrap();

    let lint = ValidRegistryDependencies::default();
    let result_d = lint.lint(&document_d);
    let result_e = lint.lint(&document_e);
    let result_f = lint.lint(&document_f);
    let result_g = lint.lint(&document_g);

    assert!(result_d.is_warning());
    assert!(result_e.is_warning());
    assert!(result_f.is_warning());
    assert!(result_g.is_warning());
}

// TODO: Add some tests with spaces
#[test]
fn lint_registry_package_dependencies_with_version_pairs() {
    macro_rules! version_pair {
        ($version_pair:literal) => {
            format!(
                "{REGISTRY_DEPENDENCY_BASE} registry:dependency \"@some/dependency {}\" .",
                $version_pair
            )
        };
    }

    // None with Any

    // Invalid
    let predicate_a_1 = version_pair!("0.1, <0.3");
    let predicate_a_2 = version_pair!("0.1 <0.3");
    let predicate_a_3 = version_pair!("0.1 <=0.3");
    let predicate_a_4 = version_pair!("<0.3 0.1");
    let predicate_a_5 = version_pair!("0.1 >0.3");
    let predicate_a_6 = version_pair!("0.1 >=0.3");
    let predicate_a_7 = version_pair!("0.1 ^0.3");
    let predicate_a_8 = version_pair!("0.1 ~0.3");

    // = with Any

    // Invalid
    let predicate_b_1 = version_pair!("=0.1, =0.3");
    let predicate_b_2 = version_pair!("=1, ^1");
    let predicate_b_3 = version_pair!("=1, ~1");
    let predicate_b_4 = version_pair!("=1, >1");
    let predicate_b_5 = version_pair!("=1, >=1");
    let predicate_b_6 = version_pair!("=1, <1");
    let predicate_b_7 = version_pair!("=1, <=1");

    // ^ with ^ or > or >= or ~
    // ~ with ~ or > or >= or ^

    // Redundant
    let predicate_c_1 = version_pair!("^1.0, ^1.1");
    let predicate_c_2 = version_pair!("^1.0.1, ~1.0.2");
    let predicate_c_3 = version_pair!("^1.0, >1.1");
    let predicate_c_4 = version_pair!("^1.0, >=1.1");
    let predicate_c_5 = version_pair!("~1.0.1, ^1.0.2");
    let predicate_c_6 = version_pair!("^1.1, ^1.2");
    let predicate_c_7 = version_pair!("^0.1, ^0.2");

    //Invalid
    let predicate_c_8 = version_pair!("~0.1.0, ~0.2.0");
    let predicate_c_9 = version_pair!("~1.0, ~1.1");
    let predicate_c_10 = version_pair!("~1.0, >1.1");
    let predicate_c_11 = version_pair!("~1.0, >=1.1");
    let predicate_c_12 = version_pair!("~1.1, ~1.2");
    let predicate_c_13 = version_pair!("^1, ^2");
    let predicate_c_14 = version_pair!("^0.1.0, ^0.2.0");
    let predicate_c_15 = version_pair!("~1, ~2");
    let predicate_c_16 = version_pair!("~0.1, ~0.2");

    // ^ with < or <=
    // ~ with < or <=

    // Valid
    let predicate_d_1 = version_pair!("~1.0.0, <1.0.5");
    let predicate_d_2 = version_pair!("~1.0.0, <=1.0.5");

    let predicate_d_3 = version_pair!("^1.0, <1.5");
    let predicate_d_4 = version_pair!("^1.0, <=1.5");

    // Invalid
    let predicate_d_5 = version_pair!("~1.0.3, <1.0.2");
    let predicate_d_6 = version_pair!("~1.0.3, <1.0.3");
    let predicate_d_7 = version_pair!("~1.0.3, <=1.0.2");
    let predicate_d_8 = version_pair!("^1.3, <1.2");
    let predicate_d_9 = version_pair!("^1.3, <1.3");
    let predicate_d_10 = version_pair!("^1.3, <=1.2");

    // > with > or >=
    // >= with >= or >

    // Redundant
    let predicate_e_1 = version_pair!(">1.0, >1.1");
    let predicate_e_2 = version_pair!(">=1.0, >1.1");

    // < with < or <=
    // <= with <= or <

    // Redundant
    let predicate_f_1 = version_pair!("<1.0, <1.1");
    let predicate_f_2 = version_pair!("<1.0, <=1.1");

    // > with < or <=
    // >= with < or <=

    // Valid
    let predicate_g_1 = version_pair!(">1.0, <1.5");
    let predicate_g_2 = version_pair!(">1.0, <=1.5");

    //  Invalid

    // TODO: Implement checking validity for these examples in the lint.

    let predicate_g_3 = version_pair!(">1.3, <1.2");
    let predicate_g_4 = version_pair!(">1.3, <1.3");
    let predicate_g_5 = version_pair!(">1.3, <=1.2");
    let predicate_g_6 = version_pair!(">1.3, <=1.3");

    // < with > or >=
    // <= with > or >=

    // Valid
    let predicate_h_1 = version_pair!(">1.0, <1.5");
    let predicate_h_2 = version_pair!(">1.0, <=1.5");
    let predicate_h_3 = version_pair!("<1.3, >=1.2");

    //  Invalid
    let predicate_h_4 = version_pair!("<1.3, >1.2");
    let predicate_h_5 = version_pair!("<1.3, >1.3");
    let predicate_h_6 = version_pair!("<1.3, >=1.3");

    let document_a_1 = TurtleDocument::parse_full(&predicate_a_1).unwrap();
    let document_a_2 = TurtleDocument::parse_full(&predicate_a_2).unwrap();
    let document_a_3 = TurtleDocument::parse_full(&predicate_a_3).unwrap();
    let document_a_4 = TurtleDocument::parse_full(&predicate_a_4).unwrap();
    let document_a_5 = TurtleDocument::parse_full(&predicate_a_5).unwrap();
    let document_a_6 = TurtleDocument::parse_full(&predicate_a_6).unwrap();
    let document_a_7 = TurtleDocument::parse_full(&predicate_a_7).unwrap();
    let document_a_8 = TurtleDocument::parse_full(&predicate_a_8).unwrap();
    let document_b_1 = TurtleDocument::parse_full(&predicate_b_1).unwrap();
    let document_b_2 = TurtleDocument::parse_full(&predicate_b_2).unwrap();
    let document_b_3 = TurtleDocument::parse_full(&predicate_b_3).unwrap();
    let document_b_4 = TurtleDocument::parse_full(&predicate_b_4).unwrap();
    let document_b_5 = TurtleDocument::parse_full(&predicate_b_5).unwrap();
    let document_b_6 = TurtleDocument::parse_full(&predicate_b_6).unwrap();
    let document_b_7 = TurtleDocument::parse_full(&predicate_b_7).unwrap();
    let document_c_1 = TurtleDocument::parse_full(&predicate_c_1).unwrap();
    let document_c_2 = TurtleDocument::parse_full(&predicate_c_2).unwrap();
    let document_c_3 = TurtleDocument::parse_full(&predicate_c_3).unwrap();
    let document_c_4 = TurtleDocument::parse_full(&predicate_c_4).unwrap();
    let document_c_5 = TurtleDocument::parse_full(&predicate_c_5).unwrap();
    let document_c_6 = TurtleDocument::parse_full(&predicate_c_6).unwrap();
    let document_c_7 = TurtleDocument::parse_full(&predicate_c_7).unwrap();
    let document_c_8 = TurtleDocument::parse_full(&predicate_c_8).unwrap();
    let document_c_9 = TurtleDocument::parse_full(&predicate_c_9).unwrap();
    let document_c_10 = TurtleDocument::parse_full(&predicate_c_10).unwrap();
    let document_c_11 = TurtleDocument::parse_full(&predicate_c_11).unwrap();
    let document_c_12 = TurtleDocument::parse_full(&predicate_c_12).unwrap();
    let document_c_13 = TurtleDocument::parse_full(&predicate_c_13).unwrap();
    let document_c_14 = TurtleDocument::parse_full(&predicate_c_14).unwrap();
    let document_c_15 = TurtleDocument::parse_full(&predicate_c_15).unwrap();
    let document_c_16 = TurtleDocument::parse_full(&predicate_c_16).unwrap();
    let document_d_1 = TurtleDocument::parse_full(&predicate_d_1).unwrap();
    let document_d_2 = TurtleDocument::parse_full(&predicate_d_2).unwrap();
    let document_d_3 = TurtleDocument::parse_full(&predicate_d_3).unwrap();
    let document_d_4 = TurtleDocument::parse_full(&predicate_d_4).unwrap();
    let document_d_5 = TurtleDocument::parse_full(&predicate_d_5).unwrap();
    let document_d_6 = TurtleDocument::parse_full(&predicate_d_6).unwrap();
    let document_d_7 = TurtleDocument::parse_full(&predicate_d_7).unwrap();
    let document_d_8 = TurtleDocument::parse_full(&predicate_d_8).unwrap();
    let document_d_9 = TurtleDocument::parse_full(&predicate_d_9).unwrap();
    let document_d_10 = TurtleDocument::parse_full(&predicate_d_10).unwrap();
    let document_e_1 = TurtleDocument::parse_full(&predicate_e_1).unwrap();
    let document_e_2 = TurtleDocument::parse_full(&predicate_e_2).unwrap();
    let document_f_1 = TurtleDocument::parse_full(&predicate_f_1).unwrap();
    let document_f_2 = TurtleDocument::parse_full(&predicate_f_2).unwrap();
    let document_g_1 = TurtleDocument::parse_full(&predicate_g_1).unwrap();
    let document_g_2 = TurtleDocument::parse_full(&predicate_g_2).unwrap();
    let document_g_3 = TurtleDocument::parse_full(&predicate_g_3).unwrap();
    let document_g_4 = TurtleDocument::parse_full(&predicate_g_4).unwrap();
    let document_g_5 = TurtleDocument::parse_full(&predicate_g_5).unwrap();
    let document_g_6 = TurtleDocument::parse_full(&predicate_g_6).unwrap();
    let document_h_1 = TurtleDocument::parse_full(&predicate_h_1).unwrap();
    let document_h_2 = TurtleDocument::parse_full(&predicate_h_2).unwrap();
    let document_h_3 = TurtleDocument::parse_full(&predicate_h_3).unwrap();
    let document_h_4 = TurtleDocument::parse_full(&predicate_h_4).unwrap();
    let document_h_5 = TurtleDocument::parse_full(&predicate_h_5).unwrap();
    let document_h_6 = TurtleDocument::parse_full(&predicate_h_6).unwrap();

    let lint = ValidRegistryDependencies::default();
    let result_a_1 = lint.lint(&document_a_1);
    let result_a_2 = lint.lint(&document_a_2);
    let result_a_3 = lint.lint(&document_a_3);
    let result_a_4 = lint.lint(&document_a_4);
    let result_a_5 = lint.lint(&document_a_5);
    let result_a_6 = lint.lint(&document_a_6);
    let result_a_7 = lint.lint(&document_a_7);
    let result_a_8 = lint.lint(&document_a_8);
    let result_b_1 = lint.lint(&document_b_1);
    let result_b_2 = lint.lint(&document_b_2);
    let result_b_3 = lint.lint(&document_b_3);
    let result_b_4 = lint.lint(&document_b_4);
    let result_b_5 = lint.lint(&document_b_5);
    let result_b_6 = lint.lint(&document_b_6);
    let result_b_7 = lint.lint(&document_b_7);
    let result_c_1 = lint.lint(&document_c_1);
    let result_c_2 = lint.lint(&document_c_2);
    let result_c_3 = lint.lint(&document_c_3);
    let result_c_4 = lint.lint(&document_c_4);
    let result_c_5 = lint.lint(&document_c_5);
    let result_c_6 = lint.lint(&document_c_6);
    let result_c_7 = lint.lint(&document_c_7);
    let result_c_8 = lint.lint(&document_c_8);
    let result_c_9 = lint.lint(&document_c_9);
    let result_c_10 = lint.lint(&document_c_10);
    let result_c_11 = lint.lint(&document_c_11);
    let result_c_12 = lint.lint(&document_c_12);
    let result_c_13 = lint.lint(&document_c_13);
    let result_c_14 = lint.lint(&document_c_14);
    let result_c_15 = lint.lint(&document_c_15);
    let result_c_16 = lint.lint(&document_c_16);
    let result_d_1 = lint.lint(&document_d_1);
    let result_d_2 = lint.lint(&document_d_2);
    let result_d_3 = lint.lint(&document_d_3);
    let result_d_4 = lint.lint(&document_d_4);
    let result_d_5 = lint.lint(&document_d_5);
    let result_d_6 = lint.lint(&document_d_6);
    let result_d_7 = lint.lint(&document_d_7);
    let result_d_8 = lint.lint(&document_d_8);
    let result_d_9 = lint.lint(&document_d_9);
    let result_d_10 = lint.lint(&document_d_10);
    let result_e_1 = lint.lint(&document_e_1);
    let result_e_2 = lint.lint(&document_e_2);
    let result_f_1 = lint.lint(&document_f_1);
    let result_f_2 = lint.lint(&document_f_2);
    let result_g_1 = lint.lint(&document_g_1);
    let result_g_2 = lint.lint(&document_g_2);
    let result_g_3 = lint.lint(&document_g_3);
    let result_g_4 = lint.lint(&document_g_4);
    let result_g_5 = lint.lint(&document_g_5);
    let result_g_6 = lint.lint(&document_g_6);
    let result_h_1 = lint.lint(&document_h_1);
    let result_h_2 = lint.lint(&document_h_2);
    let result_h_3 = lint.lint(&document_h_3);
    let result_h_4 = lint.lint(&document_h_4);
    let result_h_5 = lint.lint(&document_h_5);
    let result_h_6 = lint.lint(&document_h_6);

    // Failures
    assert!(result_a_1.is_failure());
    assert!(result_a_2.is_failure());
    assert!(result_a_3.is_failure());
    assert!(result_a_4.is_failure());
    assert!(result_a_5.is_failure());
    assert!(result_a_6.is_failure());
    assert!(result_a_7.is_failure());
    assert!(result_a_8.is_failure());
    assert!(result_b_1.is_failure());
    assert!(result_b_2.is_failure());
    assert!(result_b_3.is_failure());
    assert!(result_b_4.is_failure());
    assert!(result_b_5.is_failure());
    assert!(result_b_6.is_failure());
    assert!(result_b_7.is_failure());

    // Successes (Redundant)
    assert!(result_c_1.is_success());
    assert!(result_c_2.is_success());
    assert!(result_c_3.is_success());
    assert!(result_c_4.is_success());
    assert!(result_c_5.is_success());
    assert!(result_c_6.is_success());
    assert!(result_c_7.is_success());

    // Failures
    assert!(result_c_8.is_failure());
    assert!(result_c_9.is_failure());
    assert!(result_c_10.is_failure());
    assert!(result_c_11.is_failure());
    assert!(result_c_12.is_failure());
    assert!(result_c_13.is_failure());
    assert!(result_c_14.is_failure());
    assert!(result_c_15.is_failure());
    assert!(result_c_16.is_failure());

    // Successes
    assert!(result_d_1.is_success());
    assert!(result_d_2.is_success());
    assert!(result_d_3.is_success());
    assert!(result_d_4.is_success());
    // Failures
    assert!(result_d_5.is_failure());
    assert!(result_d_6.is_failure());
    assert!(result_d_7.is_failure());
    assert!(result_d_8.is_failure());
    assert!(result_d_9.is_failure());
    assert!(result_d_10.is_failure());
    // Success (Redundant)
    assert!(result_e_1.is_success());
    assert!(result_e_2.is_success());
    assert!(result_f_1.is_success());
    assert!(result_f_2.is_success());
    // Success
    assert!(result_g_1.is_success());
    assert!(result_g_2.is_success());
    // Failures
    assert!(result_g_3.is_failure());
    assert!(result_g_4.is_failure());
    assert!(result_g_5.is_failure());
    assert!(result_g_6.is_failure());
    // Success
    assert!(result_h_1.is_success());
    assert!(result_h_2.is_success());
    assert!(result_h_3.is_success());
    // Failures
    assert!(result_h_4.is_failure());
    assert!(result_h_5.is_failure());
    assert!(result_h_6.is_failure());
}
