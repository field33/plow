use anyhow::bail;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use pubgrub::{range::Range, version::Version};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::Serialize;
use std::str::FromStr;
use std::{convert::From, fmt};

use crate::registry::version::{SemanticVersionCompleteness, semver_completeness};

use super::IndexedDependencySpec;
use super::version::SemanticVersion;

// TODO: Do we really need serialization in this type?
// Probably it is a legacy thing which should be cleaned up.

/// A dependency type to use in [`VersionRequestResolver`](crate::resolve::VersionRequestResolver).
#[derive(Debug, Clone, Serialize)]
#[serde(rename(serialize = "DependencySpecification"))]
pub struct Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    #[serde(rename(serialize = "dependency_name"))]
    full_name: String,
    version_requirement: String,
    #[serde(skip)]
    version_range: Range<V>,
}

impl<V> Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    pub fn full_name(&self) -> &str {
        &self.full_name
    }
    pub fn version_requirement(&self) -> &str {
        &self.version_requirement
    }
    pub fn namespace_and_name(&self) -> (&str, &str) {
        self.full_name.split_once('/').unwrap()
    }

    pub fn is_version_range_none(&self) -> bool {
        self.version_range == Range::none()
    }
    pub fn is_version_range_any(&self) -> bool {
        self.version_range == Range::any()
    }

    /// Parse a version request pair and return a [`Range`](pubgrub::range::Range) which
    /// will be used later in [`VersionRequestResolver`](crate::resolve::VersionRequestResolver)
    #[allow(clippy::indexing_slicing)]
    pub fn derive_range_for_version_request_pair(predicates: &[String]) -> Result<Range<V>>
    where
        V: Version + From<SemanticVersion>,
    {
        let left_range = Self::derive_range_for_single_version_request(&predicates[0])?;
        let right_range = Self::derive_range_for_single_version_request(&predicates[1])?;
        Ok(Range::intersection(&left_range, &right_range))
    }

    /// Parse a single version request and return a [`Range`](pubgrub::range::Range) which
    /// will be used later in [`VersionRequestResolver`](crate::resolve::VersionRequestResolver)
    pub fn derive_range_for_single_version_request(version_predicate: &str) -> Result<Range<V>>
    where
        V: Version + From<SemanticVersion>,
    {
        use semver::Op::*;

        // Catch single wildcard here.
        if version_predicate == "*" {
            return Ok(Range::any());
        }

        let semantic_version: SemanticVersion = version_predicate.try_into()?;
        let parsed = semver::VersionReq::parse(version_predicate)?;
        let first_comparator = &parsed.comparators.get(0);
        if let Some(first_comparator) = first_comparator {
            let first_operator = first_comparator.op;

            return match first_operator {
                Exact => Ok(Self::exact_version_to_range(
                    first_comparator,
                    semantic_version,
                )),

                Greater => Ok(Self::greater_version_to_range(
                    first_comparator,
                    semantic_version,
                )),
                GreaterEq => Ok(Self::greater_eq_version_to_range(semantic_version)),
                Less => Ok(Self::less_version_to_range(semantic_version)),
                LessEq => Ok(Self::less_eq_version_to_range(
                    first_comparator,
                    semantic_version,
                )),
                Caret => Ok(Self::caret_version_to_range(
                    first_comparator,
                    semantic_version,
                )),
                Tilde => Ok(Self::tilde_version_to_range(
                    first_comparator,
                    semantic_version,
                )),
                // Catches only bare versions with wildcards.
                // Do not catch a single wildcard.
                // Do not catch wildcards following other operators.
                Wildcard => Ok(Self::wildcard_version_to_range(
                    first_comparator,
                    semantic_version,
                )),
                // It would be a miracle to reach this path with linted documents.
                _ => Err(anyhow!("Unsupported operator.")),
            };
        }
        Err(anyhow!("Unsupported version predicate."))
    }

    /// Convert an exact version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// =I.J.K — exactly the version I.J.K
    /// =I.J — equivalent to >=I.J.0, <I.(J+1).0
    /// =I — equivalent to >=I.0.0, <(I+1).0.0
    /// ```
    fn exact_version_to_range(comparator: &semver::Comparator, version: SemanticVersion) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        use SemanticVersionCompleteness::*;
        match semver_completeness(comparator) {
            Complete => Range::exact(version),
            OnlyMinorAndMajor => Range::between(version, version.bump_minor()),
            OnlyMajor => Range::between(version, version.bump_major()),
        }
    }

    /// Convert a greater than version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    ///  >I.J.K
    ///  >I.J — equivalent to >=I.(J+1).0
    ///  >I — equivalent to >=(I+1).0.0
    /// ```
    fn greater_version_to_range(
        comparator: &semver::Comparator,
        version: SemanticVersion,
    ) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        use SemanticVersionCompleteness::*;
        match semver_completeness(comparator) {
            Complete => Range::higher_than(version.bump_patch()),
            OnlyMinorAndMajor => Range::higher_than(version.bump_minor()),
            OnlyMajor => Range::higher_than(version.bump_major()),
        }
    }

    /// Convert a greater than or equals version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// >=I.J.K
    /// >=I.J — equivalent to >=I.J.0
    /// >=I — equivalent to >=I.0.0
    /// ```
    fn greater_eq_version_to_range(version: SemanticVersion) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        Range::higher_than(version)
    }

    /// Convert a less than version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// <I.J.K
    /// <I.J — equivalent to <I.J.0
    /// <I — equivalent to <I.0.0
    /// ```
    fn less_version_to_range(version: SemanticVersion) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        Range::strictly_lower_than(version)
    }

    /// Convert a less than or equals version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// <=I.J.K
    /// <=I.J — equivalent to <I.(J+1).0
    /// <=I — equivalent to <(I+1).0.0
    /// ```
    fn less_eq_version_to_range(
        comparator: &semver::Comparator,
        version: SemanticVersion,
    ) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        use SemanticVersionCompleteness::*;
        match semver_completeness(comparator) {
            Complete => Range::strictly_lower_than(version.bump_patch()),
            OnlyMinorAndMajor => Range::strictly_lower_than(version.bump_minor()),
            OnlyMajor => Range::strictly_lower_than(version.bump_major()),
        }
    }
    /// Convert a caret version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// ^0.J.K (for J>0) — equivalent to >=0.J.K, <0.(J+1).0
    /// ^0.0.K — equivalent to =0.0.K
    /// ^I.J.K (for I>0) — equivalent to >=I.J.K, <(I+1).0.0
    /// ^0.0 — equivalent to =0.0
    /// ^I.J (for I>0 or J>0) — equivalent to ^I.J.0
    /// ^I — equivalent to =I
    /// ```
    fn caret_version_to_range(comparator: &semver::Comparator, version: SemanticVersion) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        // =I.J — equivalent to >=I.J.0, <I.(J+1).0
        // =I — equivalent to >=I.0.0, <(I+1).0.0
        use SemanticVersionCompleteness::*;
        match semver_completeness(comparator) {
            Complete if version.is_major_zero() && !version.is_minor_zero() => {
                Range::between(version, version.bump_minor())
            }
            Complete if version.is_major_zero() && version.is_minor_zero() => Range::exact(version),
            OnlyMinorAndMajor if version.is_major_zero() && version.is_minor_zero() => {
                Range::between(
                    SemanticVersion::zero(),
                    SemanticVersion::zero().bump_minor(),
                )
            }
            Complete | OnlyMinorAndMajor | OnlyMajor => {
                Range::between(version, version.bump_major())
            }
        }
    }
    /// Convert a tilde version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// ~I.J.K — equivalent to >=I.J.K, <I.(J+1).0
    /// ~I.J — equivalent to =I.J
    /// ~I — equivalent to =I
    /// ```
    fn tilde_version_to_range(comparator: &semver::Comparator, version: SemanticVersion) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        // =I.J — equivalent to >=I.J.0, <I.(J+1).0
        // =I — equivalent to >=I.0.0, <(I+1).0.0
        use SemanticVersionCompleteness::*;
        match semver_completeness(comparator) {
            Complete | OnlyMinorAndMajor => Range::between(version, version.bump_minor()),
            OnlyMajor => Range::between(version, version.bump_major()),
        }
    }

    /// Convert a wildcard version to a [`Range`](pubgrub::range::Range)
    ///
    /// # Interpretation
    /// ```text
    /// I.J.* — equivalent to =I.J
    /// I.* or I.*.* — equivalent to =I
    /// ```
    fn wildcard_version_to_range(
        comparator: &semver::Comparator,
        version: SemanticVersion,
    ) -> Range<V>
    where
        V: Version + From<SemanticVersion>,
    {
        use SemanticVersionCompleteness::*;
        match semver_completeness(comparator) {
            #[allow(clippy::unreachable)]
            Complete => unreachable!(),
            OnlyMinorAndMajor => Range::between(version, version.bump_minor()),
            OnlyMajor => Range::between(version, version.bump_major()),
        }
    }

    /// A helper which validates field full names where the expected format is `@namespace/name` .
    ///
    /// Only alphanumeric characters and underscores are allowed.
    ///
    /// # Example
    /// ```rust,ignore
    /// assert_eq!(split_and_validate_namespace_and_name("@namespace/name").is_ok(), true);
    /// assert_eq!(split_and_validate_namespace_and_name("@name_space/name_").is_ok(), true);
    /// assert_eq!(split_and_validate_namespace_and_name("namespace/name1").is_err(), true);
    /// assert_eq!(split_and_validate_namespace_and_name("namespace").is_err(), true);
    /// assert_eq!(split_and_validate_namespace_and_name("namespace name").is_err(), true);
    /// assert_eq!(split_and_validate_namespace_and_name("namespace/hey:)").is_err(), true);
    /// ```
    pub fn split_and_validate_full_field_name(maybe_valid_full_name: &str) -> Result<(&str, &str)> {
        let (namespace, name) = maybe_valid_full_name.split_once('/').ok_or_else(|| {
            anyhow!(
                "Expected a full name in the format `@namespace/name` but got `{}`",
                maybe_valid_full_name
            )
        })?;

        if !namespace.starts_with('@') {
            return Err(anyhow!("Namespaces should be prefixed with '@'"));
        }

        if !name
            .chars()
            // Allowed chars are alphanumeric and underscores.
            .filter(|chr| *chr != '_')
            .all(char::is_alphanumeric)
            || !namespace
                .chars()
                // Allowed chars are alphanumeric, underscores and @.
                .filter(|chr| *chr != '_' && *chr != '@')
                .all(char::is_alphanumeric)
        {
            return Err(anyhow!("Only alphanumeric characters and underscores are allowed in names and an additional '@' in namespaces."));
        }

        Ok((namespace, name))
    }

    /// An  helper which validates fields where the expected value is a semantic version requirement.
    ///
    /// Everything which is possible in a semantic version requirement is allowed.
    /// Please check for the [spec](https://semver.org/) for more info.
    ///
    ///
    /// # Example
    /// ```rust,ignore
    /// assert_eq!(validate_semantic_version_requirement_literal("1.0").is_ok(), true);
    /// assert_eq!(validate_semantic_version_requirement_literal(">1.0.0").is_ok(), true);
    /// assert_eq!(validate_semantic_version_requirement_literal("^1.0.0").is_ok(), true);
    /// assert_eq!(validate_semantic_version_requirement_literal("1.0.0-alpha.1").is_ok(), true);
    /// ```
    #[allow(clippy::too_many_lines)]
    pub fn split_and_validate_semantic_version_requirement_literal(
        version_literal: &str,
    ) -> Result<Vec<String>> {
        // Empty version literal
        if version_literal.is_empty() {
            return Err(anyhow!("Version requirement literal is empty."));
        }

        // Get version literals
        let versions: Vec<String> = if version_literal.find(',').is_some() {
            // Ignore spaces and split by commas.
            version_literal
                .replace(' ', "")
                .split(',')
                .map_into()
                .collect()
        } else {
            // In the case of spaces, we go this path.
            version_literal.split(' ').map_into().collect()
        };

        // Bare version checks for both versions and pairs
        for version in &versions {
            if char::is_digit(version.chars().next().unwrap(), 10) {
                // Version is bare.
                return Err(anyhow!(
                    "Invalid version requirement, bare versions are not allowed."
                ));
            }
        }

        match versions.len() {
        // Single version
        1 => Ok(versions),

        // Version pair
        2 =>  {
            if Self::derive_range_for_version_request_pair(&versions)? == Range::none() {
                return Err(anyhow!(
                        "Impossible version range, there is no intersection between the two version requirements."
                    ));
            }

            for version in &versions {
                    // We do not allow exact operator in pairs.
                    if version.starts_with('=') {
                        return Err(anyhow!(
                            "Exact '=' operator is not allowed in version pairs."
                    ));
                }
            }

             Ok(versions)

        }
        
        _ => {
            Err(anyhow!(
                "Invalid version requirement literal, expected a version pair or a single version requirement."
            ))
        }
    }
    }

    pub fn split_string_dependency_spec(input: &str) -> Result<(String, String)> {
        let (dependency_name, mut version_requirement) = input.split(' ').enumerate().fold(
            (String::default(), String::default()),
            |(mut name, mut version_requirement), (index, part)| {
                if index > 0 {
                    version_requirement.push(' ');
                    version_requirement.push_str(part);
                } else {
                    name = part.to_owned();
                }
                (name, version_requirement)
            },
        );

        if version_requirement.is_empty() {
            bail!("Invalid dependency specification: {}", input);
        }
        // The upper if catches the possible panic.
        version_requirement.remove(0);

        if dependency_name.is_empty() || version_requirement.is_empty() {
            bail!("Invalid dependency specification: {}", input);
        }
        Ok((dependency_name, version_requirement))
    }

    pub fn try_new(dependency_name: &str, version_requirement: &str) -> Result<Self> {
        Self::split_and_validate_full_field_name(dependency_name)?;
        let version_predicates = Self::split_and_validate_semantic_version_requirement_literal(dependency_name)?;

        match version_predicates.len() {
            2 => Ok(Self {
                full_name: dependency_name.to_owned(),
                version_requirement: version_requirement.to_owned(),
                version_range: Self::derive_range_for_version_request_pair(&version_predicates)?,
            }),
            1 => Ok(Self {
                full_name: dependency_name.to_owned(),
                version_requirement: version_requirement.to_owned(),
                version_range: Self::derive_range_for_single_version_request(version_requirement)?,
            }),
            _ => Err(anyhow!(
                "Invalid version requirement: {}",
                version_requirement
            )),
        }
    }
}

impl<'de, V> Deserialize<'de> for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            DependencyName,
            VersionRequirement,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`dependency_name` or `version_requirement`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "dependency_name" => Ok(Field::DependencyName),
                            "version_requirement" => Ok(Field::VersionRequirement),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DependencyVisitor<V>
        where
            V: Version + From<SemanticVersion>,
        {
            _v: std::marker::PhantomData<V>,
        }

        impl<'de, V> Visitor<'de> for DependencyVisitor<V>
        where
            V: Version + From<SemanticVersion>,
        {
            type Value = Dependency<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Dependency")
            }

            fn visit_map<Vis>(self, mut map: Vis) -> Result<Self::Value, Vis::Error>
            where
                Vis: MapAccess<'de>,
            {
                let mut dependency_name = None;
                let mut version_requirement = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::DependencyName => {
                            if dependency_name.is_some() {
                                return Err(de::Error::duplicate_field("dependency_name"));
                            }
                            dependency_name = Some(map.next_value()?);
                        }
                        Field::VersionRequirement => {
                            if version_requirement.is_some() {
                                return Err(de::Error::duplicate_field("version_requirement"));
                            }
                            version_requirement = Some(map.next_value()?);
                        }
                    }
                }
                let dependency_name =
                    dependency_name.ok_or_else(|| de::Error::missing_field("dependency_name"))?;
                let version_requirement = version_requirement
                    .ok_or_else(|| de::Error::missing_field("version_requirement"))?;
                Dependency::try_new(dependency_name, version_requirement)
                    .map_or_else(|err| Err(de::Error::custom(format!("{}", err))), Ok)
            }
        }

        const FIELDS: &[&str] = &["dependency_name", "version_requirement"];
        deserializer.deserialize_struct(
            "Dependency",
            FIELDS,
            DependencyVisitor {
                _v: std::marker::PhantomData,
            },
        )
    }
}

impl<V> ToString for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    fn to_string(&self) -> String {
        format!("{} {}", self.full_name, self.version_requirement)
    }
}

impl<V> FromStr for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (full_name, version_requirement) = input.split_once(' ').ok_or_else(|| {
            anyhow!(
                "Input does not contain enough information to produce an `IndexedDependencySpec`."
            )
        })?;
        Self::try_new(full_name, version_requirement)
    }
}

impl<V> TryFrom<IndexedDependencySpec> for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    type Error = anyhow::Error;
    fn try_from(dependency: IndexedDependencySpec) -> Result<Self> {
        Self::try_new(&dependency.name, &dependency.req)
    }
}
// TODO: Decide necessity
// impl<V> TryFrom<PackageInLockFile> for Dependency<V>
// where
//     V: Version + From<SemanticVersion>,
// {
//     type Error = anyhow::Error;
//     fn try_from(dependency: PackageInLockFile) -> Result<Self> {
//         Self::try_new(&dependency.name, &format!("={}", dependency.version))
//     }
// }

// impl<V> TryFrom<&PackageInLockFile> for Dependency<V>
// where
//     V: Version + From<SemanticVersion>,
// {
//     type Error = anyhow::Error;
//     fn try_from(dependency: &PackageInLockFile) -> Result<Self> {
//         Self::try_new(&dependency.name, &format!("={}", dependency.version))
//     }
// }
