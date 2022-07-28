//! Contains all the supported lints.
//!
//! Lints may be anything from structural requirements on ontologies, to stylistic suggestions.
//!
//! Lints required for basic package management functionality to work (includes ontologies not meant for submission to the Field 33 registry):
//!
//! - [`BaseMatchesRootPrefix`] - Assumption taken in the `ontology_tools`; Requirement will likely be relaxed in the future
//! - [`ContainsOWLPrefixes`] - Requirement could potentially be lifted in the future, but unlikely
//! - [`ContainsRegistryPrefix`] - Assumption taken in the `ontology_tools`; Requirement will likely be relaxed in the future
//! - [`HasOntologyDeclaration`]
//! - [`HasOntologyFormatVersion`]
//! - [`HasRegistryPackageName`]
//! - [`HasRegistryPackageVersion`]
//! - [`ValidRegistryDependencies`]
//!
//!
//! Lints required for submission to the Field 33 registry (and registries following the reference implementation) - see:
//!
//! - [`RootPrefixMatchesPattern`] - To be replace with `registry:packageName` prefix with namespace requirement
//! - [`HasCanonicalPrefix`]
//!
//! Lints required for valid, tidy and clear style:
//!
//! - [`ValidRdfsLabels`]
//!
// MID PRIO
// TODO: add lint to check that there is a dependency for every defined import
// TODO: add lint to check that there is an import for every defined dependency
// TODO: add lint to check that only IRIs from direct imports (= NOT transitive imports) are used
use crate::lint::{FixSuggestion, Lint};
use harriet::{Directive, IRIReference, Item, PrefixDirective, Statement, TurtleDocument};
use plow_ontology::constants::{REGISTRY_PREFIX, REGISTRY_PREFIX_IRI};
use plow_package_management::metadata::get_root_prefix;

pub mod base_matches_root_prefix;
pub mod contains_owl_prefixes;
pub mod contains_registry_prefix;
pub mod has_canonical_prefix;
pub mod has_ontology_declaration;
pub mod has_ontology_format_version;
pub mod has_registry_author;
pub mod has_registry_category;
pub mod has_registry_package_name;
pub mod has_registry_package_version;
pub mod root_prefix_matches_pattern;
pub mod valid_rdfs_labels;
pub mod valid_registry_dependencies;

pub use base_matches_root_prefix::BaseMatchesRootPrefix;
pub use contains_owl_prefixes::ContainsOWLPrefixes;
pub use contains_registry_prefix::ContainsRegistryPrefix;
pub use has_canonical_prefix::HasCanonicalPrefix;
pub use has_ontology_declaration::HasOntologyDeclaration;
pub use has_ontology_format_version::HasOntologyFormatVersion;
pub use has_registry_author::HasRegistryAuthor;
pub use has_registry_category::HasRegistryCategory;
pub use has_registry_package_name::HasRegistryPackageName;
pub use has_registry_package_version::HasRegistryPackageVersion;
pub use root_prefix_matches_pattern::RootPrefixMatchesPattern;
pub use valid_rdfs_labels::ValidRdfsLabels;
pub use valid_registry_dependencies::ValidRegistryDependencies;

#[derive(Debug, Default, Clone)]
pub struct AddPrefixes {
    prefixes: Vec<(String, String)>,
}

impl AddPrefixes {
    pub fn for_missing_registry() -> Self {
        Self {
            prefixes: vec![(REGISTRY_PREFIX.to_owned(), REGISTRY_PREFIX_IRI.to_owned())],
        }
    }
}

impl FixSuggestion for AddPrefixes {
    fn apply(&self, document: &mut TurtleDocument) {
        let mut index_of_last_prefix = 0;
        for (i, item) in document.items.iter().enumerate() {
            if let Item::Statement(Statement::Directive(Directive::Prefix(_))) = item {
                index_of_last_prefix = i;
            }
        }
        index_of_last_prefix += 1;

        for prefix in &self.prefixes {
            let new_prefix =
                Item::Statement(Statement::Directive(Directive::Prefix(PrefixDirective {
                    prefix: Some(prefix.0.clone().into()),
                    iri: IRIReference {
                        iri: prefix.1.clone().into(),
                    },
                })));
            document.items.insert(index_of_last_prefix, new_prefix);
            index_of_last_prefix += 1;
        }
    }
}

// Currently up-casting is not existent but there are plans for it which can be followed through these links.
// https://stackoverflow.com/questions/28632968/why-doesnt-rust-support-trait-object-upcasting
// https://github.com/rust-lang/rust/issues/65991
// https://github.com/rust-lang/dyn-upcasting-coercion-initiative
#[allow(clippy::as_conversions)]
pub fn required_package_management_lints() -> Vec<Box<dyn Lint>> {
    vec![
        Box::new(BaseMatchesRootPrefix::default()) as Box<dyn Lint>,
        Box::new(ContainsOWLPrefixes::default()) as Box<dyn Lint>,
        Box::new(ContainsRegistryPrefix::default()) as Box<dyn Lint>,
        Box::new(HasOntologyDeclaration::default()) as Box<dyn Lint>,
        Box::new(HasOntologyFormatVersion::default()) as Box<dyn Lint>,
        Box::new(HasRegistryPackageName::default()) as Box<dyn Lint>,
        Box::new(HasRegistryPackageVersion::default()) as Box<dyn Lint>,
        Box::new(ValidRegistryDependencies::default()) as Box<dyn Lint>,
        Box::new(ValidRdfsLabels::default()) as Box<dyn Lint>,
    ]
}

#[allow(clippy::as_conversions)]
pub fn required_plow_registry_lints() -> Vec<Box<dyn Lint>> {
    vec![
        Box::new(BaseMatchesRootPrefix::default()) as Box<dyn Lint>,
        Box::new(ContainsOWLPrefixes::default()) as Box<dyn Lint>,
        Box::new(ContainsRegistryPrefix::default()) as Box<dyn Lint>,
        Box::new(HasOntologyDeclaration::default()) as Box<dyn Lint>,
        Box::new(HasOntologyFormatVersion::default()) as Box<dyn Lint>,
        //
        Box::new(HasRegistryPackageName::default()) as Box<dyn Lint>,
        Box::new(HasRegistryPackageVersion::default()) as Box<dyn Lint>,
        Box::new(HasRegistryAuthor::default()) as Box<dyn Lint>,
        Box::new(HasRegistryCategory::default()) as Box<dyn Lint>,
        Box::new(ValidRegistryDependencies::default()) as Box<dyn Lint>,
    ]
}

#[allow(clippy::as_conversions)]
pub fn required_style_lints() -> Vec<Box<dyn Lint>> {
    vec![Box::new(ValidRdfsLabels::default()) as Box<dyn Lint>]
}

#[allow(clippy::as_conversions)]
pub fn required_reference_registry_lints() -> Vec<Box<dyn Lint>> {
    vec![
        // Box::new(RootPrefixMatchesPattern::default()) as Box<dyn Lint>,
        Box::new(HasCanonicalPrefix::default()) as Box<dyn Lint>,
    ]
}

pub fn all_lints() -> Vec<Box<dyn Lint>> {
    let mut all = required_package_management_lints();
    let mut style_lints = required_style_lints();
    let mut reference_registry_lints = required_reference_registry_lints();
    all.append(&mut style_lints);
    all.append(&mut reference_registry_lints);
    all
}
