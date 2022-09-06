use crate::{
    lock::PackageInLockFile,
    registry::index::IndexedPackageDependency,
    version::{semver_completeness, SemanticVersion, SemanticVersionCompleteness},
};
use anyhow::bail;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use pubgrub::{range::Range, version::Version};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::Serialize;
use std::{convert::From, fmt};

/// A dependency type to use in [`VersionRequestResolver`](crate::resolve::VersionRequestResolver).
#[derive(Debug, Clone, Serialize)]
#[serde(rename(serialize = "DependencySpecification"))]
pub struct Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    #[serde(rename(serialize = "dependency_name"))]
    pub full_name: String,
    pub version_requirement: String,
    #[serde(skip)]
    pub namespace: String,
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub version_range: Range<V>,
}

impl<V> Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
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
        let (namespace, name) = dependency_name.split('/').enumerate().fold(
            (String::default(), String::default()),
            |mut namespace_and_name, (index, part)| {
                if index > 0 {
                    namespace_and_name.1 = part.to_owned();
                } else {
                    // TODO: Assuming there is an @ in the start, maybe we should keep it?
                    let mut namespace = part.to_owned();
                    namespace.remove(0);
                    namespace_and_name.0 = namespace;
                }
                namespace_and_name
            },
        );
        let version_predicates: Vec<String> = if version_requirement.find(',').is_some() {
            // Ignore spaces and split by commas.
            version_requirement
                .replace(' ', "")
                .split(',')
                .map_into()
                .collect()
        } else {
            // In the case of spaces, we go this path.
            version_requirement.split(' ').map_into().collect()
        };

        match version_predicates.len() {
            2 => Ok(Self {
                full_name: dependency_name.to_owned(),
                version_requirement: version_requirement.to_owned(),
                namespace,
                name,
                version_range: Self::derive_range_for_version_request_pair(&version_predicates)?,
            }),
            1 => Ok(Self {
                full_name: dependency_name.to_owned(),
                version_requirement: version_requirement.to_owned(),
                namespace,
                name,
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

impl<V> TryFrom<&str> for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    type Error = anyhow::Error;
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let (dependency_name, version_requirement) = Self::split_string_dependency_spec(input)?;
        Self::try_new(&dependency_name, &version_requirement)
    }
}

impl<V> TryFrom<IndexedPackageDependency> for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    type Error = anyhow::Error;
    fn try_from(dependency: IndexedPackageDependency) -> Result<Self> {
        Self::try_new(&dependency.name, &dependency.req)
    }
}

impl<V> TryFrom<PackageInLockFile> for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    type Error = anyhow::Error;
    fn try_from(dependency: PackageInLockFile) -> Result<Self> {
        Self::try_new(&dependency.name, &format!("={}", dependency.version))
    }
}

impl<V> TryFrom<&PackageInLockFile> for Dependency<V>
where
    V: Version + From<SemanticVersion>,
{
    type Error = anyhow::Error;
    fn try_from(dependency: &PackageInLockFile) -> Result<Self> {
        Self::try_new(&dependency.name, &format!("={}", dependency.version))
    }
}
