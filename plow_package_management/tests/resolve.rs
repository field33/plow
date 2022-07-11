#![allow(clippy::restriction, clippy::useless_vec)]

use plow_package_management::{
    package::OrganizationToResolveFor,
    registry::Registry,
    resolve::{Dependency, Resolver, VersionRequestResolver},
    version::semver,
    version::SemanticVersion,
};

use std::{convert::Into, str::FromStr};

use std::collections::HashMap;
use std::convert::TryFrom;

// Helper macros for tests.
// Please check how they're used in tests.
// Simple explanations included as comments.

// A dependency.
macro_rules! dep {
    ($name: literal,$version: literal) => {
        Dependency::try_from(format!("{} {}", $name, $version).as_str()).unwrap()
    };
    ($name: expr,$version: expr) => {
        Dependency::try_from(format!("{} {}", $name, $version).as_str()).unwrap()
    };
}

// A dependency.
macro_rules! deps {
    ($($dep: expr),*) => {
        vec![$($dep),*]
    };
}

// A package to add to the registry.
// Bare versions only.
macro_rules! package {
    ($pv: expr, $deps: expr) => {
        plow_package_management::package::PackageVersionWithRegistryMetadata {
            package_name: $pv.package_name,
            version: semver!(&$pv.version),
            ontology_iri: None,
            dependencies: $deps,
            cksum: None,
        }
    };
}

// Name and version of a package.
// Use it inside package! macro.
macro_rules! name_and_version {
    ($name: literal, $version: literal) => {
        plow_package_management::deps::PackageVersion {
            package_name: $name.to_string(),
            version: $version.to_string(),
        }
    };
    ($name: expr,$version: expr) => {
        plow_package_management::package::PackageVersion {
            package_name: $name.to_string(),
            version: $version.to_string(),
        }
    };
}

// A macro which defines the whole registry.
// It will insert all defined packages inside it into the registry.
macro_rules! registry {
    ($reg: expr, $($package: expr),+) => {
        $($reg.insert(
            name_and_version!($package.package_name, $package.version),
            $package,
            "".to_owned(),
        );)*
    };
}

// Runs resolution.
macro_rules! resolve_org {
    ($deps: expr, $registry: expr) => {
        Into::<VersionRequestResolver>::into(&$registry as &dyn Registry)
            .resolve_dependencies(
                OrganizationToResolveFor {
                    package_name: "@root/root".to_owned(),
                    package_version: SemanticVersion::default(),
                    dependencies: $deps,
                },
                None,
            )
            .expect("Unable to resolve dependencies")
            .packages
            .iter()
            .fold(
                HashMap::<String, SemanticVersion>::default(),
                |mut package_map, dep| {
                    package_map.insert(dep.package_name.clone(), semver!(&dep.version));
                    package_map
                },
            )
    };
}

// Runs resolution and expects to fail.
macro_rules! fail_to_resolve_org {
    ($deps: expr, $registry: expr) => {
        assert!(
            Into::<VersionRequestResolver>::into(&$registry as &dyn Registry)
                .resolve_dependencies(
                    OrganizationToResolveFor {
                        package_name: "@root/root".to_owned(),
                        package_version: SemanticVersion::default(),
                        dependencies: $deps,
                    },
                    None
                )
                .is_err()
        )
    };
}

// Get a solved package by name from the resolution results.
macro_rules! solved {
    ($package_version_map: expr, $name: literal) => {
        *$package_version_map.get($name).unwrap()
    };
    ($package_version_map: expr, $name: expr) => {
        *$package_version_map.get($name).unwrap()
    };
}

// Cities as dependencies for testing.
const BERLIN: &str = "@cities/Berlin";
const FRANKFURT: &str = "@cities/Frankfurt";
const HAMBURG: &str = "@cities/Hamburg";
const MAINZ: &str = "@cities/Mainz";

#[test]
fn resolutions_with_no_transitive_deps() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();

    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.0.1"),
            deps![dep!(FRANKFURT, "=0.0.1"), dep!(HAMBURG, "=0.0.1")]
        ),
        package!(name_and_version!(FRANKFURT, "0.0.1"), deps![]),
        package!(name_and_version!(HAMBURG, "0.0.1"), deps![])
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=0.0.1")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("0.0.1"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.0.1"));
    assert_eq!(solved!(solution, HAMBURG), semver!("0.0.1"));

    //

    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.1.0"),
            deps![dep!(FRANKFURT, "=0.1"), dep!(HAMBURG, "=0.0.1")]
        ),
        package!(name_and_version!(FRANKFURT, "0.1.1"), deps![]),
        package!(name_and_version!(FRANKFURT, "0.1.2"), deps![]),
        package!(name_and_version!(FRANKFURT, "0.2.1"), deps![]),
        package!(name_and_version!(HAMBURG, "0.0.1"), deps![])
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=0.1.0")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("0.1.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.1.2"));
    assert_eq!(solved!(solution, HAMBURG), semver!("0.0.1"));

    //

    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "1.5.5"),
            deps![dep!(FRANKFURT, "1.x")]
        ),
        package!(
            name_and_version!(BERLIN, "1.9.2"),
            deps![dep!(FRANKFURT, "1.x")]
        ),
        package!(name_and_version!(FRANKFURT, "1.1.1"), deps![]),
        package!(name_and_version!(FRANKFURT, "1.3.2"), deps![]),
        package!(name_and_version!(FRANKFURT, "1.9.12"), deps![])
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=1.5.5")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("1.5.5"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("1.9.12"));
}

#[test]
fn resolutions_with_simple_transitive_deps() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(FRANKFURT, "=0.0.1")]
        ),
        package!(
            name_and_version!(FRANKFURT, "0.0.1"),
            deps![dep!(HAMBURG, "=0.0.1")]
        ),
        package!(
            name_and_version!(HAMBURG, "0.0.1"),
            deps![dep!(BERLIN, "=0.2.0")]
        )
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("0.2.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.0.1"));
    assert_eq!(solved!(solution, HAMBURG), semver!("0.0.1"));
}

#[test]
fn resolutions_with_complex_transitive_deps() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "1.0.0"),
            deps![dep!(FRANKFURT, "^0.1")]
        ),
        package!(name_and_version!(BERLIN, "1.5.0"), deps![]),
        package!(
            name_and_version!(FRANKFURT, "0.1.3"),
            deps![dep!(HAMBURG, "3.*")]
        ),
        package!(name_and_version!(FRANKFURT, "0.0.3"), deps![]),
        package!(name_and_version!(FRANKFURT, "0.0.12"), deps![]),
        package!(
            name_and_version!(HAMBURG, "3.5.6"),
            deps![dep!(BERLIN, "<1.7")]
        ),
        package!(
            name_and_version!(HAMBURG, "3.5.1"),
            deps![dep!(BERLIN, "<1.6")]
        ),
        package!(
            name_and_version!(MAINZ, "2.1.0"),
            deps![dep!(FRANKFURT, "0.1.x"), dep!(BERLIN, "=1.0.0")]
        )
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=1.0.0")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("1.0.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.1.3"));
    assert_eq!(solved!(solution, HAMBURG), semver!("3.5.6"));

    let solution = resolve_org!(deps![dep!(BERLIN, "^1")], registry);
    assert_eq!(solved!(solution, BERLIN), semver!("1.5.0"));

    let solution = resolve_org!(deps![dep!(HAMBURG, ">=3"), dep!(MAINZ, "2.1.*")], registry);
    assert_eq!(solved!(solution, BERLIN), semver!("1.0.0"));
    assert_eq!(solved!(solution, HAMBURG), semver!("3.5.6"));
    assert_eq!(solved!(solution, MAINZ), semver!("2.1.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.1.3"));
}

#[test]
fn resolutions_with_cyclic_deps() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(FRANKFURT, "=0.0.1")]
        ),
        package!(
            name_and_version!(FRANKFURT, "0.0.1"),
            deps![dep!(BERLIN, "=0.2.0")]
        )
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);
    assert_eq!(solved!(solution, BERLIN), semver!("0.2.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.0.1"));

    let solution = resolve_org!(deps![dep!(FRANKFURT, "=0.0.1")], registry);
    assert_eq!(solved!(solution, BERLIN), semver!("0.2.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.0.1"));

    let solution = resolve_org!(
        deps![dep!(FRANKFURT, "=0.0.1"), dep!(BERLIN, "=0.2.0")],
        registry
    );
    assert_eq!(solved!(solution, BERLIN), semver!("0.2.0"));
    assert_eq!(solved!(solution, FRANKFURT), semver!("0.0.1"));
}

#[test]
fn resolutions_with_self_dependence_fails() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(BERLIN, "=0.2.0")]
        )
    );

    fail_to_resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);
    fail_to_resolve_org!(deps![dep!(BERLIN, "=0.*")], registry);
}

#[test]
#[should_panic]
fn resolutions_with_invalid_dependencies_fail() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(HAMBURG, ">0.2.0")]
        ),
        package!(name_and_version!(HAMBURG, "0.2.0"), deps![])
    );

    fail_to_resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);

    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(name_and_version!(BERLIN, "0.3.0"), deps![]),
        package!(name_and_version!(BERLIN, "1.3.0"), deps![]),
        package!(name_and_version!(BERLIN, "0.3.5"), deps![])
    );

    fail_to_resolve_org!(deps![dep!(BERLIN, ">0.3.5")], registry);
}

#[test]
fn resolutions_with_incompatible_dependencies_fail() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(HAMBURG, "=0.2.0"), dep!(FRANKFURT, "=0.2.0")]
        ),
        package!(
            name_and_version!(HAMBURG, "0.2.0"),
            deps![dep!(MAINZ, ">=1.0.0")]
        ),
        package!(
            name_and_version!(FRANKFURT, "0.2.0"),
            deps![dep!(MAINZ, "<1.0.0")]
        ),
        package!(name_and_version!(MAINZ, "0.5.0"), deps![]),
        package!(name_and_version!(MAINZ, "1.0.0"), deps![])
    );

    fail_to_resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);

    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(HAMBURG, "=0.2.0"), dep!(FRANKFURT, "=0.2.0")]
        ),
        package!(
            name_and_version!(HAMBURG, "0.2.0"),
            deps![dep!(MAINZ, "=0.1.1")]
        ),
        package!(
            name_and_version!(FRANKFURT, "0.2.0"),
            deps![dep!(MAINZ, "=0.1.0")]
        ),
        package!(name_and_version!(MAINZ, "0.1.0"), deps![]),
        package!(name_and_version!(MAINZ, "0.1.1"), deps![])
    );
    fail_to_resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);
}

#[test]
fn resolutions_which_have_multiple_versions_of_the_same_package_fail() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(HAMBURG, "=0.2.0"), dep!(FRANKFURT, "=0.2.0")]
        ),
        package!(
            name_and_version!(HAMBURG, "0.2.0"),
            deps![dep!(MAINZ, "=0.1.0")]
        ),
        package!(
            name_and_version!(FRANKFURT, "0.2.0"),
            deps![dep!(MAINZ, "=0.1.1")]
        ),
        package!(name_and_version!(MAINZ, "0.1.0"), deps![]),
        package!(name_and_version!(MAINZ, "0.1.1"), deps![])
    );

    fail_to_resolve_org!(deps![dep!(BERLIN, "0.2.0")], registry);
}

#[test]
fn resolutions_with_version_request_pairs() {
    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(HAMBURG, ">0.0.1 <0.0.5")]
        ),
        package!(name_and_version!(HAMBURG, "0.0.1"), deps![]),
        package!(name_and_version!(HAMBURG, "0.0.2"), deps![]),
        package!(name_and_version!(HAMBURG, "0.0.3"), deps![]),
        package!(name_and_version!(HAMBURG, "0.0.4"), deps![]),
        package!(name_and_version!(HAMBURG, "0.0.5"), deps![])
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("0.2.0"));
    assert_eq!(solved!(solution, HAMBURG), semver!("0.0.4"));

    let mut registry = plow_package_management::registry::in_memory::InMemoryRegistry::default();
    registry!(
        registry,
        package!(
            name_and_version!(BERLIN, "0.2.0"),
            deps![dep!(HAMBURG, "^1.0, ^1.1")]
        ),
        package!(name_and_version!(HAMBURG, "1.0.0"), deps![]),
        package!(name_and_version!(HAMBURG, "1.1.0"), deps![]),
        package!(name_and_version!(HAMBURG, "1.2.0"), deps![]),
        package!(name_and_version!(HAMBURG, "1.3.0"), deps![]),
        package!(name_and_version!(HAMBURG, "1.4.0"), deps![]),
        package!(name_and_version!(HAMBURG, "1.4.1"), deps![])
    );

    let solution = resolve_org!(deps![dep!(BERLIN, "=0.2.0")], registry);

    assert_eq!(solved!(solution, BERLIN), semver!("0.2.0"));
    assert_eq!(solved!(solution, HAMBURG), semver!("1.4.1"));
}
