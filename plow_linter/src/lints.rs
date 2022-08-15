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
use harriet::{Directive, IRIReference, PrefixDirective, Statement, TurtleDocument};
use plow_ontology::constants::{REGISTRY_PREFIX, REGISTRY_PREFIX_IRI};
use plow_package_management::metadata::get_root_prefix;

pub mod base_matches_root_prefix;
pub mod contains_owl_prefixes;
pub mod contains_registry_prefix;
pub mod exists_registry_license;
pub mod exists_registry_license_spdx;
pub mod has_at_least_one_valid_license_annotation;
pub mod has_canonical_prefix;
pub mod has_ontology_declaration;
pub mod has_ontology_format_version;
pub mod has_rdfs_comment_manifest_context;
pub mod has_rdfs_label_manifest_context;
pub mod has_registry_author;
pub mod has_registry_category;
pub mod has_registry_keyword;
pub mod has_registry_license;
pub mod has_registry_license_spdx;
pub mod has_registry_package_name;
pub mod has_registry_package_version;
pub mod has_registry_short_description;
pub mod root_prefix_matches_pattern;
pub mod valid_rdfs_labels;
pub mod valid_registry_dependencies;
pub mod valid_registry_documentation;
pub mod valid_registry_homepage;
pub mod valid_registry_repository;

pub use base_matches_root_prefix::BaseMatchesRootPrefix;
pub use contains_owl_prefixes::ContainsOWLPrefixes;
pub use contains_registry_prefix::ContainsRegistryPrefix;
pub use exists_registry_license::ExistsRegistryLicense;
pub use exists_registry_license_spdx::ExistsRegistryLicenseSPDX;
pub use has_at_least_one_valid_license_annotation::HasAtLeastOneValidLicenseAnnotation;
pub use has_canonical_prefix::HasCanonicalPrefix;
pub use has_ontology_declaration::HasOntologyDeclaration;
pub use has_ontology_format_version::HasOntologyFormatVersion;
pub use has_rdfs_comment_manifest_context::HasRdfsCommentManifestContext;
pub use has_rdfs_label_manifest_context::HasRdfsLabelManifestContext;
pub use has_registry_author::HasRegistryAuthor;
pub use has_registry_category::HasRegistryCategory;
pub use has_registry_keyword::HasRegistryKeyword;
pub use has_registry_license::HasRegistryLicense;
pub use has_registry_license_spdx::HasRegistryLicenseSPDX;
pub use has_registry_package_name::HasRegistryPackageName;
pub use has_registry_package_version::HasRegistryPackageVersion;
pub use has_registry_short_description::HasRegistryShortDescription;
pub use root_prefix_matches_pattern::RootPrefixMatchesPattern;
pub use valid_rdfs_labels::ValidRdfsLabels;
pub use valid_registry_dependencies::ValidRegistryDependencies;
pub use valid_registry_documentation::ValidRegistryDocumentation;
pub use valid_registry_homepage::ValidRegistryHomepage;
pub use valid_registry_repository::ValidRegistryRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintName {
    BaseMatchesRootPrefix,
    ContainsOWLPrefixes,
    ContainsRegistryPrefix,
    ExistsRegistryLicense,
    ExistsRegistryLicenseSPDX,
    HasAtLeastOneValidLicenseAnnotation,
    HasCanonicalPrefix,
    HasOntologyDeclaration,
    HasOntologyFormatVersion,
    HasRdfsCommentManifestContext,
    HasRdfsLabelManifestContext,
    HasRegistryAuthor,
    HasRegistryCategory,
    HasRegistryKeyword,
    HasRegistryLicense,
    HasRegistryLicenseSPDX,
    HasRegistryPackageName,
    HasRegistryPackageVersion,
    HasRegistryShortDescription,
    RootPrefixMatchesPattern,
    ValidRdfsLabels,
    ValidRegistryDependencies,
    ValidRegistryDocumentation,
    ValidRegistryHomepage,
    ValidRegistryRepository,
}

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
        for (i, statement) in document.statements.iter().enumerate() {
            if let Statement::Directive(Directive::Prefix(_)) = statement {
                index_of_last_prefix = i;
            }
        }
        index_of_last_prefix += 1;

        for prefix in &self.prefixes {
            let new_prefix = Statement::Directive(Directive::Prefix(PrefixDirective {
                leading_whitespace: None,
                prefix: Some(prefix.0.clone().into()),
                iri: IRIReference {
                    iri: prefix.1.clone().into(),
                },
            }));
            document.statements.insert(index_of_last_prefix, new_prefix);
            index_of_last_prefix += 1;
        }
    }
}

pub type PlowLint = Box<dyn Lint + Send + Sync + 'static>;

pub struct LintSet {
    pub lints: Vec<PlowLint>,
    pub sub_lints: Option<Vec<PlowLint>>,
}

impl LintSet {
    pub fn new(lints: Vec<PlowLint>, sub_lints: Option<Vec<PlowLint>>) -> Self {
        Self { lints, sub_lints }
    }
}

pub fn all_lints() -> LintSet {
    let mut all_lints = required_plow_registry_lints();
    all_lints.extend(required_style_lints());
    let all_sub_lints = required_plow_registry_sub_lints();
    LintSet::new(all_lints, Some(all_sub_lints))
}

pub fn lints_for_field_submission() -> LintSet {
    let lints = required_plow_registry_lints();
    let sub_lints = Some(required_plow_registry_sub_lints());
    LintSet::new(lints, sub_lints)
}

// TODO: This part needs some order and more organization.
// It could be done in a later PR but we need to organize and not duplicate lints here.

// Currently up-casting is not existent but there are plans for it which can be followed through these links.
// https://stackoverflow.com/questions/28632968/why-doesnt-rust-support-trait-object-upcasting
// https://github.com/rust-lang/rust/issues/65991
// https://github.com/rust-lang/dyn-upcasting-coercion-initiative
#[allow(clippy::as_conversions)]
fn required_base_lints() -> Vec<PlowLint> {
    vec![
        Box::new(BaseMatchesRootPrefix::default()) as PlowLint,
        Box::new(ContainsOWLPrefixes::default()) as PlowLint,
        Box::new(ContainsRegistryPrefix::default()) as PlowLint,
        Box::new(HasOntologyDeclaration::default()) as PlowLint,
        Box::new(HasOntologyFormatVersion::default()) as PlowLint,
    ]
}

#[allow(clippy::as_conversions)]
fn required_plow_registry_lints() -> Vec<Box<dyn Lint + Send + Sync>> {
    let mut plow_registry_lints = required_base_lints();
    plow_registry_lints.extend(vec![
        Box::new(HasRegistryPackageName::default()) as PlowLint,
        Box::new(HasRegistryPackageVersion::default()) as PlowLint,
        Box::new(HasRegistryAuthor::default()) as PlowLint,
        Box::new(HasRegistryCategory::default()) as PlowLint,
        Box::new(HasRegistryKeyword::default()) as PlowLint,
        Box::new(HasAtLeastOneValidLicenseAnnotation::default()) as PlowLint,
        Box::new(ValidRegistryDependencies::default()) as PlowLint,
        Box::new(ValidRegistryHomepage::default()) as PlowLint,
        Box::new(ValidRegistryRepository::default()) as PlowLint,
        Box::new(ValidRegistryDocumentation::default()) as PlowLint,
        Box::new(HasRegistryShortDescription::default()) as PlowLint,
        Box::new(HasRdfsCommentManifestContext::default()) as PlowLint,
        Box::new(HasRdfsLabelManifestContext::default()) as PlowLint,
        // TODO: Re-check these to see if we need them and why we need them.
        // Box::new(RootPrefixMatchesPattern::default()) as PlowLint,
        // Box::new(HasCanonicalPrefix::default()) as PlowLint,
    ]);
    plow_registry_lints
}

#[allow(clippy::as_conversions)]
fn required_plow_registry_sub_lints() -> Vec<Box<dyn Lint + Send + Sync>> {
    vec![
        Box::new(ExistsRegistryLicense::default()) as PlowLint,
        Box::new(ExistsRegistryLicenseSPDX::default()) as PlowLint,
        Box::new(HasRegistryLicense::default()) as PlowLint,
        Box::new(HasRegistryLicenseSPDX::default()) as PlowLint,
    ]
}

#[allow(clippy::as_conversions)]
fn required_style_lints() -> Vec<Box<dyn Lint + Send + Sync>> {
    vec![Box::new(ValidRdfsLabels::default()) as PlowLint]
}
