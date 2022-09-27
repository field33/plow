use anyhow::{Error, Result};
use pubgrub::range::Range;
use semver::VersionReq;
use std::convert::From;
use std::{fmt::Display, str::FromStr};

/// Macro to quickly generate a [`SemanticVersion`] from a string.
///
/// # Panics
/// This macro uses expect to panic if the string is not a valid semantic version.
#[macro_export]
macro_rules! semver {
    ($version:literal) => {
        SemanticVersion::from_str($version).expect("Not a valid semantic version string.")
    };
    ($version:expr) => {
        SemanticVersion::from_str($version).expect("Not a valid semantic version string.")
    };
}
/// An extension of [`Range`](pubgrub::range::Range) trait which allows extra
/// functionality for comparing semantic version ranges.
pub trait RangeExt<V>
where
    V: pubgrub::version::Version + From<SemanticVersion>,
{
    /// If true, version pair is redundant, pick the larger range.
    fn subset_of(&self, other: &Range<V>) -> bool;
    /// If true, version pair is invalid, just feed the empty range.
    fn possible(&self, other: &Range<V>) -> bool;
}

impl RangeExt<SemanticVersion> for Range<SemanticVersion> {
    fn subset_of(&self, other: &Self) -> bool {
        Self::intersection(self, other) == *self
    }
    fn possible(&self, other: &Self) -> bool {
        Self::intersection(self, other) != Self::none()
    }
}

/// Represents the completeness of a semantic version.
///
/// `1` would be [`OnlyMajor`](SemanticVersionCompleteness::OnlyMajor), `1.2`
/// would be [`MajorAndMinor`](SemanticVersionCompleteness::MajorAndMinor)
/// and `1.2.3` would be [`Complete`](SemanticVersionCompleteness::Complete)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticVersionCompleteness {
    OnlyMajor,
    OnlyMinorAndMajor,
    Complete,
}

/// Checks the completeness of a semantic version.
///
/// # Examples
/// ```rust
/// # use plow_package_management::version::{semver_completeness,SemanticVersionCompleteness};
/// let complete = &semver::VersionReq::parse("1.2.3").unwrap().comparators[0];
/// let major_minor = &semver::VersionReq::parse("1.2").unwrap().comparators[0];
/// let only_major = &semver::VersionReq::parse("1").unwrap().comparators[0];
///
/// use SemanticVersionCompleteness::*;
/// assert_eq!(Complete, semver_completeness(complete));
/// assert_eq!(OnlyMinorAndMajor, semver_completeness(major_minor));
/// assert_eq!(OnlyMajor, semver_completeness(only_major));
/// ```
pub const fn semver_completeness(comparator: &semver::Comparator) -> SemanticVersionCompleteness {
    use SemanticVersionCompleteness::*;
    if comparator.minor.is_some() && comparator.patch.is_some() {
        return Complete;
    }
    if comparator.minor.is_some() && comparator.patch.is_none() {
        return OnlyMinorAndMajor;
    }
    OnlyMajor
}

/// Our own semantic version type to use in pubgrub based resolver.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl serde::Serialize for SemanticVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> serde::Deserialize<'de> for SemanticVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl SemanticVersion {
    /// Create a version with "major", "minor" and "patch" values.
    /// `version = major.minor.patch`
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn as_sum(&self) -> u64 {
        self.major + self.minor + self.patch
    }

    /// Version 0.0.0.
    pub const fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    #[must_use]
    /// Bump the patch number of a version.
    pub const fn bump_patch(self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }

    #[must_use]
    /// Bump the minor number of a version.
    pub const fn bump_minor(self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    #[must_use]
    /// Bump the major number of a version.
    pub const fn bump_major(self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    #[must_use]
    /// Returns `true` if the patch field of the version is 0.
    pub const fn is_patch_zero(&self) -> bool {
        self.patch == 0
    }

    #[must_use]
    /// Returns `true` if the minor field of the version is 0.
    pub const fn is_minor_zero(&self) -> bool {
        self.minor == 0
    }

    #[must_use]
    /// Returns `true` if the major field of the version is 0.
    pub const fn is_major_zero(&self) -> bool {
        self.major == 0
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// Convert a tuple (major, minor, patch) into a version.
impl From<(u64, u64, u64)> for SemanticVersion {
    fn from(tuple: (u64, u64, u64)) -> Self {
        let (major, minor, patch) = tuple;
        Self::new(major, minor, patch)
    }
}

// Convert a version into a tuple (major, minor, patch).
impl From<SemanticVersion> for (u64, u64, u64) {
    fn from(v: SemanticVersion) -> Self {
        (v.major, v.minor, v.patch)
    }
}

#[allow(clippy::indexing_slicing)]
impl TryFrom<&str> for SemanticVersion {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let req = VersionReq::parse(s)?;

        Ok(Self {
            // bare,
            major: req.comparators[0].major,
            minor: req.comparators[0].minor.unwrap_or(0),
            patch: req.comparators[0].patch.unwrap_or(0),
        })
    }
}

#[allow(clippy::indexing_slicing)]
impl TryFrom<String> for SemanticVersion {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(&s)
    }
}

#[allow(clippy::indexing_slicing)]
impl TryFrom<&String> for SemanticVersion {
    type Error = Error;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[allow(clippy::indexing_slicing)]
impl FromStr for SemanticVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

// Implement the trait `Version` for us to use this type with pubgrub's DependencyProvider trait.
impl pubgrub::version::Version for SemanticVersion {
    fn lowest() -> Self {
        Self::zero()
    }
    fn bump(&self) -> Self {
        self.bump_patch()
    }
}

pub use semver;
