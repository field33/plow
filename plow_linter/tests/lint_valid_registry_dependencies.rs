use plow_linter::lints::{PlowLint, ValidRegistryDependencies};
use plow_linter::Linter;

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

    let mut linter_a = Linter::try_from(ttl_document_with_registry_dependency_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_registry_dependency_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_registry_dependency_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_registry_dependency_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_registry_dependency_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();

    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_success());
    assert!(result_c.first().unwrap().is_failure());
    assert!(result_d.first().unwrap().is_success());
    assert!(result_e.first().unwrap().is_failure());
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

    let mut linter_d = Linter::try_from(ttl_document_with_registry_dependency_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_registry_dependency_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_f = Linter::try_from(ttl_document_with_registry_dependency_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g = Linter::try_from(ttl_document_with_registry_dependency_g.as_ref()).unwrap();
    linter_g.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );

    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();
    let result_g = linter_g.run_all_lints();

    assert!(result_d.first().unwrap().is_warning());
    assert!(result_e.first().unwrap().is_warning());
    assert!(result_f.first().unwrap().is_warning());
    assert!(result_g.first().unwrap().is_warning());
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

    let mut linter_a_1 = Linter::try_from(predicate_a_1.as_ref()).unwrap();
    linter_a_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_2 = Linter::try_from(predicate_a_2.as_ref()).unwrap();
    linter_a_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_3 = Linter::try_from(predicate_a_3.as_ref()).unwrap();
    linter_a_3.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_4 = Linter::try_from(predicate_a_4.as_ref()).unwrap();
    linter_a_4.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_5 = Linter::try_from(predicate_a_5.as_ref()).unwrap();
    linter_a_5.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_6 = Linter::try_from(predicate_a_6.as_ref()).unwrap();
    linter_a_6.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_7 = Linter::try_from(predicate_a_7.as_ref()).unwrap();
    linter_a_7.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_a_8 = Linter::try_from(predicate_a_8.as_ref()).unwrap();
    linter_a_8.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_1 = Linter::try_from(predicate_b_1.as_ref()).unwrap();
    linter_b_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_2 = Linter::try_from(predicate_b_2.as_ref()).unwrap();
    linter_b_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_3 = Linter::try_from(predicate_b_3.as_ref()).unwrap();
    linter_b_3.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_4 = Linter::try_from(predicate_b_4.as_ref()).unwrap();
    linter_b_4.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_5 = Linter::try_from(predicate_b_5.as_ref()).unwrap();
    linter_b_5.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_6 = Linter::try_from(predicate_b_6.as_ref()).unwrap();
    linter_b_6.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_b_7 = Linter::try_from(predicate_b_7.as_ref()).unwrap();
    linter_b_7.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_1 = Linter::try_from(predicate_c_1.as_ref()).unwrap();
    linter_c_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_2 = Linter::try_from(predicate_c_2.as_ref()).unwrap();
    linter_c_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_3 = Linter::try_from(predicate_c_3.as_ref()).unwrap();
    linter_c_3.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_4 = Linter::try_from(predicate_c_4.as_ref()).unwrap();
    linter_c_4.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_5 = Linter::try_from(predicate_c_5.as_ref()).unwrap();
    linter_c_5.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_6 = Linter::try_from(predicate_c_6.as_ref()).unwrap();
    linter_c_6.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_7 = Linter::try_from(predicate_c_7.as_ref()).unwrap();
    linter_c_7.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_8 = Linter::try_from(predicate_c_8.as_ref()).unwrap();
    linter_c_8.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_9 = Linter::try_from(predicate_c_9.as_ref()).unwrap();
    linter_c_9.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_10 = Linter::try_from(predicate_c_10.as_ref()).unwrap();
    linter_c_10.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_11 = Linter::try_from(predicate_c_11.as_ref()).unwrap();
    linter_c_11.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_12 = Linter::try_from(predicate_c_12.as_ref()).unwrap();
    linter_c_12.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_13 = Linter::try_from(predicate_c_13.as_ref()).unwrap();
    linter_c_13.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_14 = Linter::try_from(predicate_c_14.as_ref()).unwrap();
    linter_c_14.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_15 = Linter::try_from(predicate_c_15.as_ref()).unwrap();
    linter_c_15.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_c_16 = Linter::try_from(predicate_c_16.as_ref()).unwrap();
    linter_c_16.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_1 = Linter::try_from(predicate_d_1.as_ref()).unwrap();
    linter_d_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_2 = Linter::try_from(predicate_d_2.as_ref()).unwrap();
    linter_d_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_3 = Linter::try_from(predicate_d_3.as_ref()).unwrap();
    linter_d_3.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_4 = Linter::try_from(predicate_d_4.as_ref()).unwrap();
    linter_d_4.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_5 = Linter::try_from(predicate_d_5.as_ref()).unwrap();
    linter_d_5.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_6 = Linter::try_from(predicate_d_6.as_ref()).unwrap();
    linter_d_6.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_7 = Linter::try_from(predicate_d_7.as_ref()).unwrap();
    linter_d_7.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_8 = Linter::try_from(predicate_d_8.as_ref()).unwrap();
    linter_d_8.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_9 = Linter::try_from(predicate_d_9.as_ref()).unwrap();
    linter_d_9.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_d_10 = Linter::try_from(predicate_d_10.as_ref()).unwrap();
    linter_d_10.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_e_1 = Linter::try_from(predicate_e_1.as_ref()).unwrap();
    linter_e_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_e_2 = Linter::try_from(predicate_e_2.as_ref()).unwrap();
    linter_e_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_f_1 = Linter::try_from(predicate_f_1.as_ref()).unwrap();
    linter_f_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_f_2 = Linter::try_from(predicate_f_2.as_ref()).unwrap();
    linter_f_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g_1 = Linter::try_from(predicate_g_1.as_ref()).unwrap();
    linter_g_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g_2 = Linter::try_from(predicate_g_2.as_ref()).unwrap();
    linter_g_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g_3 = Linter::try_from(predicate_g_3.as_ref()).unwrap();
    linter_g_3.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g_4 = Linter::try_from(predicate_g_4.as_ref()).unwrap();
    linter_g_4.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g_5 = Linter::try_from(predicate_g_5.as_ref()).unwrap();
    linter_g_5.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_g_6 = Linter::try_from(predicate_g_6.as_ref()).unwrap();
    linter_g_6.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_h_1 = Linter::try_from(predicate_h_1.as_ref()).unwrap();
    linter_h_1.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_h_2 = Linter::try_from(predicate_h_2.as_ref()).unwrap();
    linter_h_2.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_h_3 = Linter::try_from(predicate_h_3.as_ref()).unwrap();
    linter_h_3.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_h_4 = Linter::try_from(predicate_h_4.as_ref()).unwrap();
    linter_h_4.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_h_5 = Linter::try_from(predicate_h_5.as_ref()).unwrap();
    linter_h_5.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );
    let mut linter_h_6 = Linter::try_from(predicate_h_6.as_ref()).unwrap();
    linter_h_6.add_lint_as_set(
        vec![Box::new(ValidRegistryDependencies::default()) as PlowLint],
        None,
    );

    let result_a_1 = linter_a_1.run_all_lints();
    let result_a_2 = linter_a_2.run_all_lints();
    let result_a_3 = linter_a_3.run_all_lints();
    let result_a_4 = linter_a_4.run_all_lints();
    let result_a_5 = linter_a_5.run_all_lints();
    let result_a_6 = linter_a_6.run_all_lints();
    let result_a_7 = linter_a_7.run_all_lints();
    let result_a_8 = linter_a_8.run_all_lints();
    let result_b_1 = linter_b_1.run_all_lints();
    let result_b_2 = linter_b_2.run_all_lints();
    let result_b_3 = linter_b_3.run_all_lints();
    let result_b_4 = linter_b_4.run_all_lints();
    let result_b_5 = linter_b_5.run_all_lints();
    let result_b_6 = linter_b_6.run_all_lints();
    let result_b_7 = linter_b_7.run_all_lints();
    let result_c_1 = linter_c_1.run_all_lints();
    let result_c_2 = linter_c_2.run_all_lints();
    let result_c_3 = linter_c_3.run_all_lints();
    let result_c_4 = linter_c_4.run_all_lints();
    let result_c_5 = linter_c_5.run_all_lints();
    let result_c_6 = linter_c_6.run_all_lints();
    let result_c_7 = linter_c_7.run_all_lints();
    let result_c_8 = linter_c_8.run_all_lints();
    let result_c_9 = linter_c_9.run_all_lints();
    let result_c_10 = linter_c_10.run_all_lints();
    let result_c_11 = linter_c_11.run_all_lints();
    let result_c_12 = linter_c_12.run_all_lints();
    let result_c_13 = linter_c_13.run_all_lints();
    let result_c_14 = linter_c_14.run_all_lints();
    let result_c_15 = linter_c_15.run_all_lints();
    let result_c_16 = linter_c_16.run_all_lints();
    let result_d_1 = linter_d_1.run_all_lints();
    let result_d_2 = linter_d_2.run_all_lints();
    let result_d_3 = linter_d_3.run_all_lints();
    let result_d_4 = linter_d_4.run_all_lints();
    let result_d_5 = linter_d_5.run_all_lints();
    let result_d_6 = linter_d_6.run_all_lints();
    let result_d_7 = linter_d_7.run_all_lints();
    let result_d_8 = linter_d_8.run_all_lints();
    let result_d_9 = linter_d_9.run_all_lints();
    let result_d_10 = linter_d_10.run_all_lints();
    let result_e_1 = linter_e_1.run_all_lints();
    let result_e_2 = linter_e_2.run_all_lints();
    let result_f_1 = linter_f_1.run_all_lints();
    let result_f_2 = linter_f_2.run_all_lints();
    let result_g_1 = linter_g_1.run_all_lints();
    let result_g_2 = linter_g_2.run_all_lints();
    let result_g_3 = linter_g_3.run_all_lints();
    let result_g_4 = linter_g_4.run_all_lints();
    let result_g_5 = linter_g_5.run_all_lints();
    let result_g_6 = linter_g_6.run_all_lints();
    let result_h_1 = linter_h_1.run_all_lints();
    let result_h_2 = linter_h_2.run_all_lints();
    let result_h_3 = linter_h_3.run_all_lints();
    let result_h_4 = linter_h_4.run_all_lints();
    let result_h_5 = linter_h_5.run_all_lints();
    let result_h_6 = linter_h_6.run_all_lints();

    // Failures
    assert!(result_a_1.first().unwrap().is_failure());
    assert!(result_a_2.first().unwrap().is_failure());
    assert!(result_a_3.first().unwrap().is_failure());
    assert!(result_a_4.first().unwrap().is_failure());
    assert!(result_a_5.first().unwrap().is_failure());
    assert!(result_a_6.first().unwrap().is_failure());
    assert!(result_a_7.first().unwrap().is_failure());
    assert!(result_a_8.first().unwrap().is_failure());
    assert!(result_b_1.first().unwrap().is_failure());
    assert!(result_b_2.first().unwrap().is_failure());
    assert!(result_b_3.first().unwrap().is_failure());
    assert!(result_b_4.first().unwrap().is_failure());
    assert!(result_b_5.first().unwrap().is_failure());
    assert!(result_b_6.first().unwrap().is_failure());
    assert!(result_b_7.first().unwrap().is_failure());

    // Successes (Redundant)
    assert!(result_c_1.first().unwrap().is_success());
    assert!(result_c_2.first().unwrap().is_success());
    assert!(result_c_3.first().unwrap().is_success());
    assert!(result_c_4.first().unwrap().is_success());
    assert!(result_c_5.first().unwrap().is_success());
    assert!(result_c_6.first().unwrap().is_success());
    assert!(result_c_7.first().unwrap().is_success());

    // Failures
    assert!(result_c_8.first().unwrap().is_failure());
    assert!(result_c_9.first().unwrap().is_failure());
    assert!(result_c_10.first().unwrap().is_failure());
    assert!(result_c_11.first().unwrap().is_failure());
    assert!(result_c_12.first().unwrap().is_failure());
    assert!(result_c_13.first().unwrap().is_failure());
    assert!(result_c_14.first().unwrap().is_failure());
    assert!(result_c_15.first().unwrap().is_failure());
    assert!(result_c_16.first().unwrap().is_failure());

    // Successes
    assert!(result_d_1.first().unwrap().is_success());
    assert!(result_d_2.first().unwrap().is_success());
    assert!(result_d_3.first().unwrap().is_success());
    assert!(result_d_4.first().unwrap().is_success());
    // Failures
    assert!(result_d_5.first().unwrap().is_failure());
    assert!(result_d_6.first().unwrap().is_failure());
    assert!(result_d_7.first().unwrap().is_failure());
    assert!(result_d_8.first().unwrap().is_failure());
    assert!(result_d_9.first().unwrap().is_failure());
    assert!(result_d_10.first().unwrap().is_failure());
    // Success (Redundant)
    assert!(result_e_1.first().unwrap().is_success());
    assert!(result_e_2.first().unwrap().is_success());
    assert!(result_f_1.first().unwrap().is_success());
    assert!(result_f_2.first().unwrap().is_success());
    // Success
    assert!(result_g_1.first().unwrap().is_success());
    assert!(result_g_2.first().unwrap().is_success());
    // Failures
    assert!(result_g_3.first().unwrap().is_failure());
    assert!(result_g_4.first().unwrap().is_failure());
    assert!(result_g_5.first().unwrap().is_failure());
    assert!(result_g_6.first().unwrap().is_failure());
    // Success
    assert!(result_h_1.first().unwrap().is_success());
    assert!(result_h_2.first().unwrap().is_success());
    assert!(result_h_3.first().unwrap().is_success());
    // Failures
    assert!(result_h_4.first().unwrap().is_failure());
    assert!(result_h_5.first().unwrap().is_failure());
    assert!(result_h_6.first().unwrap().is_failure());
}
