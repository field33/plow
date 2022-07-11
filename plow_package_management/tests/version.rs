#![allow(clippy::all, clippy::restriction)]

use plow_package_management::resolve::dependency::Dependency;
use plow_package_management::version::{
    semver_completeness, SemanticVersion, SemanticVersionCompleteness,
};
use pubgrub::range::Range;
use std::str::FromStr;

macro_rules! range {
    ($version:literal) => {
        Dependency::<SemanticVersion>::derive_range_for_single_version_request($version).unwrap()
    };
}
macro_rules! semver {
    ($version:literal) => {
        SemanticVersion::from_str($version).expect("Not a valid semantic version string.")
    };
}

#[test]
fn version_to_range_exact() {
    assert_eq!(range!("=1"), Range::between(semver!("1"), semver!("2")));
    assert_eq!(
        range!("=1.1"),
        Range::between(semver!("1.1"), semver!("1.2"))
    );
    assert_eq!(range!("=1.1.1"), Range::exact(semver!("1.1.1")));
}
#[test]
fn version_to_range_greater() {
    assert_eq!(range!(">1"), Range::higher_than(semver!("2")));
    assert_eq!(range!(">1.1"), Range::higher_than(semver!("1.2")));
    assert_eq!(range!(">1.1.1"), Range::higher_than(semver!("1.1.2")));
}
#[test]
fn version_to_range_greater_eq() {
    assert_eq!(range!(">=1"), Range::higher_than(semver!("1")));
    assert_eq!(range!(">=1.1"), Range::higher_than(semver!("1.1")));
    assert_eq!(range!(">=1.1.1"), Range::higher_than(semver!("1.1.1")));
}
#[test]
fn version_to_range_less() {
    assert_eq!(range!("<1"), Range::strictly_lower_than(semver!("1")));
    assert_eq!(range!("<1.1"), Range::strictly_lower_than(semver!("1.1")));
    assert_eq!(
        range!("<1.1.1"),
        Range::strictly_lower_than(semver!("1.1.1"))
    );
}
#[test]
fn version_to_range_less_eq() {
    assert_eq!(range!("<=1"), Range::strictly_lower_than(semver!("2")));
    assert_eq!(range!("<=1.1"), Range::strictly_lower_than(semver!("1.2")));
    assert_eq!(
        range!("<=1.1.1"),
        Range::strictly_lower_than(semver!("1.1.2"))
    );
}

#[test]
fn version_to_range_caret() {
    assert_eq!(range!("^1"), Range::between(semver!("1"), semver!("2")));
    assert_eq!(
        range!("^1.1"),
        Range::between(semver!("1.1"), semver!("2.0"))
    );
    assert_eq!(
        range!("^0.0"),
        Range::between(semver!("0.0"), semver!("0.1"))
    );
    assert_eq!(
        range!("^1.1.1"),
        Range::between(semver!("1.1.1"), semver!("2.0.0"))
    );
    assert_eq!(range!("^0.0.1"), Range::exact(semver!("0.0.1")));
    assert_eq!(
        range!("^0.1.1"),
        Range::between(semver!("0.1.1"), semver!("0.2.0"))
    );
    assert_eq!(
        range!("^0.2.0"),
        Range::between(semver!("0.2.0"), semver!("0.3.0"))
    );
}

#[test]
fn version_to_range_tilde() {
    assert_eq!(range!("~1"), Range::between(semver!("1"), semver!("2")));
    assert_eq!(
        range!("~1.1"),
        Range::between(semver!("1.1"), semver!("1.2"))
    );
    assert_eq!(
        range!("~1.1.1"),
        Range::between(semver!("1.1.1"), semver!("1.2.0"))
    );
}
#[test]
fn version_to_range_wildcard() {
    assert_eq!(range!("*"), Range::any());
    assert_eq!(
        range!("1.1.*"),
        Range::between(semver!("1.1"), semver!("1.2"))
    );
    assert_eq!(range!("1.*"), Range::between(semver!("1"), semver!("2")));
    assert_eq!(range!("1.*.*"), Range::between(semver!("1"), semver!("2")));
    assert_eq!(
        range!("1.1.x"),
        Range::between(semver!("1.1"), semver!("1.2"))
    );
    assert_eq!(range!("1.x"), Range::between(semver!("1"), semver!("2")));
    assert_eq!(range!("1.x.x"), Range::between(semver!("1"), semver!("2")));
}

#[test]
fn test_semver_completeness() {
    let v1 = semver_completeness(&semver::VersionReq::parse("=1").unwrap().comparators[0]);
    let v2 = semver_completeness(&semver::VersionReq::parse("=1.1").unwrap().comparators[0]);
    let v3 = semver_completeness(&semver::VersionReq::parse("=1.1.1").unwrap().comparators[0]);
    assert_eq!(SemanticVersionCompleteness::OnlyMajor, v1);
    assert_eq!(SemanticVersionCompleteness::OnlyMinorAndMajor, v2);
    assert_eq!(SemanticVersionCompleteness::Complete, v3);
}
