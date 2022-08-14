// TODO: Will be refactored for the updated linter later.

// use harriet::TurtleDocument;
// use plow_ontology::initialize_ontology;
// use plow_linter::lint::{Fixes, Lint, LintResult};
// use plow_linter::lints::{required_package_management_lints, required_reference_registry_lints};

// #[allow(unused)]
// struct CheckedLint {
//     lint: Box<dyn Lint>,
//     result: LintResult,
//     suggested_fixes: Vec<Fixes>,
// }

// #[test]
// fn initialized_ontology_is_pkg_mgmt_conformant() -> Result<(), anyhow::Error> {
//     let ontology = initialize_ontology("@namespace/package_name")?;
//     let document = TurtleDocument::parse_full(&ontology).unwrap();

//     let lints = required_package_management_lints();
//     let mut checked_lints = Vec::new();
//     for lint in lints {
//         let suggested_fixes = lint.suggest_fix(&document).unwrap_or_default();

//         checked_lints.push(CheckedLint {
//             result: lint.lint(&document),
//             suggested_fixes,
//             lint,
//         });
//     }

//     dbg!(&checked_lints
//         .iter()
//         .map(|n| n.result.clone())
//         .collect::<Vec<_>>());
//     let all_lints_passed = checked_lints.iter().all(|lint| lint.result.is_success());
//     assert!(all_lints_passed);
//     Ok(())
// }

// #[test]
// fn initialized_ontology_is_field33_registry_conformant() -> Result<(), anyhow::Error> {
//     let ontology = initialize_ontology("@namespace/package_name")?;
//     let document = TurtleDocument::parse_full(&ontology).unwrap();

//     let lints = required_reference_registry_lints();
//     let mut checked_lints = Vec::new();
//     for lint in lints {
//         let suggested_fixes = lint.suggest_fix(&document).unwrap_or_default();

//         checked_lints.push(CheckedLint {
//             result: lint.lint(&document),
//             suggested_fixes,
//             lint,
//         });
//     }

//     dbg!(&checked_lints
//         .iter()
//         .map(|n| n.result.clone())
//         .collect::<Vec<_>>());
//     let all_lints_passed = checked_lints.iter().all(|lint| lint.result.is_success());
//     assert!(all_lints_passed);
//     Ok(())
// }
