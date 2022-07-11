use crate::lint::{lint_failure, LintResult};
use anyhow::Result;
use plow_package_management::resolve::Dependency;
use plow_package_management::version::SemanticVersion;
use rdftk_core::model::statement::Statement;
use semver::Version;
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use thiserror::Error;

/// An internal helper which catches the absence of single annotations which must exist.
pub fn catch_single_annotations_which_must_exist(
    annotations: &HashSet<&Rc<dyn Statement>>,
    related_field: &str,
) -> Option<LintResult> {
    if annotations.is_empty() {
        return Some(lint_failure!(&format!(
            "No `{related_field}` annotations found."
        )));
    }
    if annotations.len() > 1 {
        return Some(lint_failure!(&format!(
            "More than 1 `{related_field}` annotations found."
        )));
    }
    None
}

#[derive(Error, Debug)]
pub enum NamespaceAndNameLintError {
    #[error(
        "should be alphanumeric. Only underscores ('_') as an additional character is allowed."
    )]
    InvalidFormat,
    #[error("should be in the form of `@<namespace>/<package-name>`")]
    InvalidChars,
}

/// An internal helper which validates fields where the expected format is `@namespace/name` .
///
/// Only alphanumeric characters and underscores are allowed.
///
/// # Example
/// ```rust,ignore
/// assert_eq!(validate_namespace_and_name("@namespace/name").is_ok(), true);
/// assert_eq!(validate_namespace_and_name("@name_space/name_").is_ok(), true);
/// assert_eq!(validate_namespace_and_name("namespace/name1").is_err(), true);
/// assert_eq!(validate_namespace_and_name("namespace").is_err(), true);
/// assert_eq!(validate_namespace_and_name("namespace name").is_err(), true);
/// assert_eq!(validate_namespace_and_name("namespace/hey:)").is_err(), true);
/// ```
pub fn validate_namespace_and_name(
    namespace_and_name_literal: &str,
) -> Result<(), NamespaceAndNameLintError> {
    let mut namespace_and_name: Vec<_> = namespace_and_name_literal.split('/').collect();

    if let Some(namespace) = namespace_and_name.get(0) {
        if !namespace.starts_with('@') {
            return Err(NamespaceAndNameLintError::InvalidFormat);
        }
        if namespace_and_name.len() != 2 {
            return Err(NamespaceAndNameLintError::InvalidFormat);
        }
    } else {
        return Err(NamespaceAndNameLintError::InvalidFormat);
    }

    // We can unwrap here because we already checked that the length is 2
    // Filter underscores and '@' since they are the only allowed chars other than alphanumeric ones.
    #[allow(clippy::unwrap_used)]
    let name = namespace_and_name.pop().unwrap().replace('_', "");

    #[allow(clippy::unwrap_used)]
    let namespace = namespace_and_name
        .pop()
        .unwrap()
        .replace('_', "")
        .replace('@', "");

    if !name.chars().all(char::is_alphanumeric) || !namespace.chars().all(char::is_alphanumeric) {
        return Err(NamespaceAndNameLintError::InvalidChars);
    }

    Ok(())
}

#[derive(Error, Debug, Clone)]
pub enum VersionLiteralLintFailureOrWarning {
    #[error(transparent)]
    Failure(VersionLiteralLintFailure),
    #[error(transparent)]
    Warning(VersionLiteralLintWarning),
}

#[derive(Error, Debug, Clone)]
pub enum VersionLiteralLintFailure {
    #[error("does not allow for empty version literals")]
    Empty,
    #[error("does not allow for pre-release or build indicators, e.g. `1.0.0-alpha.1+001`")]
    PreReleaseOrBuildNotAllowed,
    #[error("does not allow versions other than a single version or a pair of versions.")]
    OnlySingleOrPair,
    #[error("does not allow version pairs with `=` character in it.")]
    NoExactPrefixOnVersionPairs,
    #[error("can not be solved.")]
    CanNotBeSolved,
    #[error("does not allow for bare versions, e.g. `1.0.0`. The version which is bare is {0}")]
    BareVersionNotAllowed(String),
    #[error("is not valid: {0}\nPlease avoid prefixes and make your version complete, including all `major`, `minor` and `patch` fields.")]
    InvalidSemanticVersionLiteral(String),
}

#[derive(Error, Debug, Clone)]
pub enum VersionLiteralLintWarning {
    #[error("contains a single wildcard `*` it might be better to be more specific.")]
    ContainsSingleWildcard,
    #[error("contains one or more wildcards `*` or `x` it might be better to be more specific. The version which contains wildcards is: {0}")]
    ContainsWildcards(String),
}

/// An internal helper which validates fields where the expected value is a version
/// and the expected format is `major.minor.patch` .
///
/// Only simple and fully complete version strings are allowed
/// with no prefix or suffixes. e.g. major.minor.patch.
///
/// # Example
/// ```rust,ignore
/// assert_eq!(validate_version_literal_conservative("1.0.0").is_ok(), true);
/// assert_eq!(validate_version_literal_conservative(">1.0.0").is_err(), true);
/// assert_eq!(validate_version_literal_conservative("^1.0.0").is_err(), true);
/// assert_eq!(validate_version_literal_conservative("1.0.0-alpha.1").is_err(), true);
/// ```
pub fn validate_semantic_version_literal(
    version_literal: &str,
) -> Result<(), VersionLiteralLintFailureOrWarning> {
    // Check if the version string is valid
    use VersionLiteralLintFailureOrWarning::*;
    Version::parse(version_literal)
        .map_err(|err| {
            Failure(VersionLiteralLintFailure::InvalidSemanticVersionLiteral(
                err.to_string(),
            ))
        })
        .and_then(|parsed| {
            if parsed.pre.is_empty() && parsed.build.is_empty() {
                Ok(())
            } else {
                Err(Failure(
                    VersionLiteralLintFailure::PreReleaseOrBuildNotAllowed,
                ))
            }
        })
}

/// An internal helper which validates fields where the expected value is a semantic version.
///
/// Everything which is possible in a semantic version is allowed.
/// Please check for the [spec](https://semver.org/) for more info.
///
///
/// # Example
/// ```rust,ignore
/// assert_eq!(validate_version_literal_permissive("1.0").is_ok(), true);
/// assert_eq!(validate_version_literal_permissive(">1.0.0").is_ok(), true);
/// assert_eq!(validate_version_literal_permissive("^1.0.0").is_ok(), true);
/// assert_eq!(validate_version_literal_permissive("1.0.0-alpha.1").is_ok(), true);
/// ```
#[allow(clippy::too_many_lines)]
pub fn validate_semantic_version_requirement_literal(
    version_literal: &str,
) -> Result<(), Vec<VersionLiteralLintFailureOrWarning>> {
    use VersionLiteralLintFailureOrWarning::*;

    let failures_and_warnings: RefCell<Vec<VersionLiteralLintFailureOrWarning>> =
        RefCell::new(vec![]);

    // Empty version literal
    if version_literal.is_empty() {
        failures_and_warnings
            .borrow_mut()
            .push(Failure(VersionLiteralLintFailure::Empty));
        return Err(failures_and_warnings.take());
    }

    // Get version literals
    let versions: Vec<&str> = version_literal
        .split(|character| character == ',' || character == ' ')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    // Bare version and wildcard checks for both versions and pairs
    for version in &versions {
        let has_wildcards = version.contains('*') || version.contains('x');
        #[allow(clippy::else_if_without_else)]
        if has_wildcards {
            // Has wildcards, omit bare version check.
            failures_and_warnings.borrow_mut().push(Warning(
                VersionLiteralLintWarning::ContainsWildcards((*version_literal).to_owned()),
            ));
            // We know that literals are not empty here.
        } else if char::is_digit(version.chars().next().expect("Unreachable"), 10) {
            // Version is bare.
            failures_and_warnings.borrow_mut().push(Failure(
                VersionLiteralLintFailure::BareVersionNotAllowed((*version_literal).to_owned()),
            ));
        } else if *version == "*" {
            // If we have a single wildcard a version can be bare but we give a warning.
            failures_and_warnings
                .borrow_mut()
                .push(Warning(VersionLiteralLintWarning::ContainsSingleWildcard));
        }
    }

    match versions.len() {
        // Single version
        1 => {
            if failures_and_warnings.borrow().is_empty() {
                return Ok(());
            }
            Err(failures_and_warnings.take())
        }
        // Version pairs
        //
        // We're using the Dependency type a bit out of purpose here.
        // On the other hand in the construction phase it has complicated
        // logic implemented to derive a version range. This eases checking
        // validity of version pairs a lot.
        2 => Dependency::<SemanticVersion>::try_new("@dummy/dummy", version_literal).map_or_else(
            |err| {
                failures_and_warnings.borrow_mut().push(Failure(
                    VersionLiteralLintFailure::InvalidSemanticVersionLiteral(err.to_string()),
                ));
                Err(failures_and_warnings.take())
            },
            |d: Dependency<SemanticVersion>| {
                // Checks if there is an intersection between individual version requirements in a pair.
                if d.is_version_range_none() {
                    failures_and_warnings
                        .borrow_mut()
                        .push(Failure(VersionLiteralLintFailure::CanNotBeSolved));
                }

                for version in &versions {
                    // We do not allow exact operator in pairs.
                    if version.starts_with('=') {
                        failures_and_warnings.borrow_mut().push(Failure(
                            VersionLiteralLintFailure::NoExactPrefixOnVersionPairs,
                        ));
                    }
                }

                if failures_and_warnings.borrow().is_empty() {
                    return Ok(());
                }
                Err(failures_and_warnings.take())
            },
        ),
        _ => {
            failures_and_warnings
                .borrow_mut()
                .push(Failure(VersionLiteralLintFailure::OnlySingleOrPair));
            Err(failures_and_warnings.take())
        }
    }
}
