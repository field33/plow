pub mod dependency;
pub mod error;

pub use crate::version::semver;
pub use dependency::Dependency;

use crate::{
    lock::PackageInLockFile,
    package::{
        OrganizationToResolveFor, PackageSet, PackageVersion, PackageVersionWithRegistryMetadata,
    },
    registry::Registry,
    version::SemanticVersion,
    ORGANIZATION, ORGANIZATION_NAME, ORGANIZATION_VERSION,
};

use pubgrub::{
    range::Range,
    report::Reporter,
    solver::{DependencyConstraints, DependencyProvider},
};

use std::{
    borrow::Borrow,
    collections::BTreeMap,
    convert::{From, Into},
};

use anyhow::Result;
use fallible_iterator::FallibleIterator;
use itertools::Itertools;

use self::error::ResolverError;
use std::str::FromStr;

/// A trait for different dependency resolver implementations.
pub trait Resolver {
    /// Resolves a set of requested dependencies to a set of package versions.
    fn resolve_dependencies(
        &self,
        organization_to_resolve_for: OrganizationToResolveFor,
        locked_dependencies: Option<&[PackageInLockFile]>,
    ) -> Result<PackageSet, ResolverError>;
}

/// A type which is used in resolver to collect all available versions of a package.
pub type SemanticVersionAndRangePair = (SemanticVersion, Range<SemanticVersion>);

/// A type which is used in resolver to collect all available versions of a package.
pub type AvailablePackagesWithDepConstraints =
    BTreeMap<SemanticVersion, DependencyConstraints<String, SemanticVersion>>;

/// A type which is used in resolver to collect valid versions of a package to be picked later.
pub type ValidPackagesIntermediateCache = BTreeMap<String, Vec<SemanticVersion>>;

/// A cache to use across resolver iterations.
pub type AvailablePackagesCache = BTreeMap<String, Vec<PackageVersionWithRegistryMetadata>>;
/// A resolver which uses pubgrub algorithm to resolve dependencies with semantic version requests.
pub struct VersionRequestResolver<'req_resolver> {
    // Access to registry.
    registry: &'req_resolver dyn Registry,
    // A cache to use across a single resolver iteration.
    valid_versions_intermediate_cache: std::cell::RefCell<ValidPackagesIntermediateCache>,
    // A cache to use across resolver iterations.
    available_packages_cache: std::cell::RefCell<AvailablePackagesCache>,
    locked_dependencies: std::cell::RefCell<Option<BTreeMap<String, SemanticVersionAndRangePair>>>,
}

impl<'req_resolver> From<&'req_resolver dyn Registry> for VersionRequestResolver<'req_resolver> {
    fn from(registry: &'req_resolver dyn Registry) -> Self {
        Self {
            registry,
            valid_versions_intermediate_cache: std::cell::RefCell::new(
                ValidPackagesIntermediateCache::default(),
            ),
            available_packages_cache: std::cell::RefCell::new(AvailablePackagesCache::default()),
            locked_dependencies: std::cell::RefCell::new(None),
        }
    }
}

impl<'req_resolver> VersionRequestResolver<'req_resolver> {
    /// Handles the errors originate directly from dependency resolution.
    fn handle_resolution_errors(
        error: pubgrub::error::PubGrubError<String, SemanticVersion>,
    ) -> ResolverError {
        // Error in resolution
        use pubgrub::error::PubGrubError::*;
        match error {
            NoSolution(mut tree) => {
                tree.collapse_no_versions();
                let report = pubgrub::report::DefaultStringReporter::report(&tree);
                ResolverError::SolutionError(report.replace(ORGANIZATION, "organization").replace(
                    "Organization is forbidden",
                    "Dependencies of the organization could not be resolved",
                ))
            }
            ErrorRetrievingDependencies {
                package,
                version,
                source,
            } => {
                let mut message = format!("Error retrieving dependencies for package {package} at version {version}.\n Error: {source}");
                message = message.replace(
                    &format!("package {ORGANIZATION_NAME} at version {ORGANIZATION_VERSION}"),
                    "organization",
                );
                ResolverError::SolutionError(message)
            }
            DependencyOnTheEmptySet {
                package,
                version,
                dependent,
            } => {
                let mut message = format!("Returned a dependency on an empty range.\nThe package {dependent} requires us to pick from an empty set.\nThe package which was being tried to be picked: name {package}, version {version}.");
                message = message.replace(
                    &format!("package {ORGANIZATION_NAME} at version {ORGANIZATION_VERSION}"),
                    "organization",
                );
                ResolverError::SolutionError(message)
            }
            SelfDependency { package, version } => {
                let mut message = format!("Self dependency detected.\nThe package {package} requires itself at version {version}.");
                message = message.replace(
                    &format!("The package {ORGANIZATION_NAME} requires itself at version {ORGANIZATION_VERSION}"),
                    "Organization can not depend on itself",
                );
                ResolverError::SolutionError(message)
            }
            ErrorChoosingPackageVersion(err) => {
                ResolverError::SolutionError(format!("Error choosing package version.\n{err}"))
            }
            ErrorInShouldCancel(err) => {
                ResolverError::SolutionError(format!("Error in should_cancel.\n{err}"))
            }
            Failure(reason) => ResolverError::SolutionError(reason),
        }
    }

    /// Post processes the resolved dependencies.
    ///
    /// This includes removing duplicates, sorting and validating if there are multiple
    /// versions of the same package.
    fn post_process_and_deliver_resolved_dependencies(
        &self,
        resolved_dependencies: Vec<PackageVersion>,
    ) -> Result<PackageSet, ResolverError> {
        // Here we apply any post processing to resolved dependencies
        let packages: Vec<PackageVersion> = resolved_dependencies
            .into_iter()
            // Filter the root package which represents the organization which we resolve for.
            .filter(|package| package.package_name != ORGANIZATION_NAME)
            // Sort
            .sorted()
            // Remove duplicates
            .dedup()
            .collect();

        // Collect packages which are the same but have multiple different versions for a possible future error message.
        let same_packages_with_multiple_versions: Vec<&PackageVersion> = packages
            .iter()
            .filter(|package_checking| {
                packages.iter().any(|package_comparing| {
                    // Note: Duplicates are removed at this point

                    // Same package name
                    package_checking.package_name == package_comparing.package_name
                            // Different versions
                                && package_checking.version != package_comparing.version
                            // Other than the package we're currently checking.
                                && package_checking != &package_comparing
                })
            })
            .collect();

        if same_packages_with_multiple_versions.is_empty() {
            // Clear the cache.
            self.available_packages_cache.borrow_mut().clear();
            // Solved
            return Ok(PackageSet { packages });
        }

        // There are multiple versions of same packages so we should inform the user about it with an error.
        let error_message = same_packages_with_multiple_versions.into_iter().fold(
                    "Packages or dependencies may not have multiple versions of the same package.\nThe packages which have multiple versions defined are:\n"
                        .to_owned(),
                    |mut message, package| {
                        message
                            .push_str(&format!("\t{} {}\n", package.package_name, package.version));
                        message
                    },
                );
        Err(ResolverError::SolutionError(error_message))
    }

    fn get_valid_packages<U: std::borrow::Borrow<Range<SemanticVersion>>>(
        available_packages: &[PackageVersionWithRegistryMetadata],
        requested_range: &U,
    ) -> Result<Vec<SemanticVersion>, ResolverError> {
        fallible_iterator::convert(available_packages.iter()
        .try_fold(
            AvailablePackagesWithDepConstraints::default(),
            |mut available_versions_of_package_and_their_dependency_constraints,
             metadata|
             -> Result<AvailablePackagesWithDepConstraints, ResolverError> {
                // For the value of the map we need to create a new DependencyConstraints type.
                // For this we need to get all the dependencies of this package.
                let dependencies = metadata.dependencies.iter().try_fold(
                    DependencyConstraints::<String, SemanticVersion>::default(),
                    |mut dependencies,
                     dep|
                     -> Result<
                        DependencyConstraints<String, SemanticVersion>,
                        ResolverError,
                    > {
                        dependencies.insert(dep.full_name.clone(), dep.version_range.clone());
                        Ok(dependencies)
                    },
                );
                match dependencies {
                    Ok(dependencies) => {
                        // Collect all available packages.
                        available_versions_of_package_and_their_dependency_constraints
                            .insert(metadata.version, dependencies);
                        Ok(available_versions_of_package_and_their_dependency_constraints)
                    }
                    Err(err) => Err(err),
                }
            },
        )?
        // Transform the retrieval to the type we need.
        .borrow()
        .keys()
        .sorted()
        .into_iter()
        .copied().map(Ok))
        // We give the constraint here.
        .filter(|v| Ok(requested_range.borrow().contains(v.borrow())))
        .collect()
    }

    /// Counts and caches the valid versions of a package.
    ///
    /// By valid version we mean that the version of a package
    /// which falls in to the provided version requirement range.
    fn cache_and_count_valid_versions_of_package<
        T: std::borrow::Borrow<String>,
        U: std::borrow::Borrow<Range<SemanticVersion>>,
    >(
        &self,
        available_packages: &[PackageVersionWithRegistryMetadata],
        requested_package: &T,
        requested_range: &U,
    ) -> Result<usize, ResolverError> {
        let valid_packages = Self::get_valid_packages(available_packages, requested_range)?;

        // Quantity of valid versions of a package
        let count = valid_packages.len();

        // Store them in an intermediate cache for the picking operation for this run, indexed by package name.
        // This is a step to share the state out of the closure calling this function.
        self.valid_versions_intermediate_cache
            .borrow_mut()
            .insert(requested_package.borrow().clone(), valid_packages);

        Ok(count)
    }
}

impl<'req_resolver> Resolver for VersionRequestResolver<'req_resolver> {
    /// Resolves a set of requested dependencies to a set of package versions
    fn resolve_dependencies(
        &self,
        organization_to_resolve_for: OrganizationToResolveFor,
        locked_dependencies: Option<&[PackageInLockFile]>,
        // TODO: A user requested update will be added here.
    ) -> Result<PackageSet, ResolverError> {
        // In our use case we solve for organizations but not packages.
        // An organization has dependencies but itself is not a package but more of an entity.
        // You can not depend on an organization or add it as a dependency.
        // Organization can depend on packages and it is always our entry point to the resolution.
        // To cover for this case we always start with 1 item in our cache, our root organization which we solve for.
        // This way the resolver will find it in the first hit and continue resolving real packages.
        // Store the organization in the cache.
        self.available_packages_cache.borrow_mut().insert(
            organization_to_resolve_for.package_name.clone(),
            vec![PackageVersionWithRegistryMetadata {
                package_name: organization_to_resolve_for.package_name.clone(),
                version: organization_to_resolve_for.package_version,
                ontology_iri: None,
                dependencies: organization_to_resolve_for.dependencies,
                cksum: None,
            }],
        );

        // Collect locked dependencies if there are some.
        if let Some(packages) = locked_dependencies {
            *self.locked_dependencies.borrow_mut() =
                Some(
                    packages.iter().try_fold(
                        BTreeMap::new(),
                        |mut locked_dependencies,
                         package|
                         -> Result<
                            BTreeMap<String, SemanticVersionAndRangePair>,
                            ResolverError,
                        > {
                            let intermediate = Dependency::<SemanticVersion>::try_from(package)
                                .map_err(|err| ResolverError::InvalidLockFile(err.to_string()))?;
                            locked_dependencies.insert(
                                intermediate.full_name,
                                (
                                    semver!(&intermediate.version_requirement),
                                    intermediate.version_range,
                                ),
                            );
                            Ok(locked_dependencies)
                        },
                    )?,
                );
        }

        // Now we are ready to start the resolution.
        pubgrub::solver::resolve(
            self,
            organization_to_resolve_for.package_name,
            organization_to_resolve_for.package_version,
        )
        .map_or_else(Err, |solved| {
            // Transform to PackageVersion structs.
            Ok(solved.iter().map_into().collect::<Vec<PackageVersion>>())
        })
        .map_or_else(
            |err| Err(Self::handle_resolution_errors(err)),
            |resolved_dependencies| {
                // Post process and deliver
                self.post_process_and_deliver_resolved_dependencies(resolved_dependencies)
            },
        )
    }
}

impl<'req_resolver> DependencyProvider<String, SemanticVersion>
    for VersionRequestResolver<'req_resolver>
{
    // Chooses a valid package  version for the given package.
    // There may be multiple versions of the package in focus.
    // We first find all available versions of a package and pick the highest version in our specified constrains.

    // Unwraps used here will not panic and they're explained.
    #[allow(clippy::unwrap_in_result)]
    // I sincerely think that the first match in this function is the most readable way to write it.
    #[allow(clippy::single_match_else)]
    fn choose_package_version<
        T: std::borrow::Borrow<String>,
        U: std::borrow::Borrow<Range<SemanticVersion>>,
    >(
        &self,
        potential_packages: impl Iterator<Item = (T, U)>,
    ) -> Result<(T, Option<SemanticVersion>), Box<dyn std::error::Error>> {
        // This function is a re-implementation of the pubgrub `choose_package_with_fewest_versions` function with some minor changes.
        // You may check the original at (https://github.com/pubgrub-rs/pubgrub/blob/717289be5722dd5caaa0d1f4ed13047d11a7f7fd/src/solver.rs#L279-L300)
        let count_valid = |(package, range): &(T, U)| -> Result<usize, ResolverError> {
            // Early return if a package in potential_packages hits the lock file,

            // TODO: A way to update individual packages in the lock file.
            // Ignore this for the package if it has an update flag.
            // Change package type to include update flags.
            if let Some(ref locked_dependencies) = *self.locked_dependencies.borrow() {
                if let Some((locked_dependency_version, locked_dependency_range)) =
                    locked_dependencies.get(package.borrow())
                {
                    if locked_dependency_range.intersection(range.borrow()) != Range::none() {
                        self.valid_versions_intermediate_cache.borrow_mut().insert(
                            package.borrow().clone(),
                            // We know that this is always a valid, single, complete and exact version.
                            vec![*locked_dependency_version],
                        );

                        // Pick only one valid version since we exactly want that version.
                        return Ok(1);
                    }
                }
            }

            // TODO: In future iterations we'll add other resources for checking for dependencies such as the local file system.
            // Currently we only check registry.

            // Check if we already retrieved this information and stored it in cache.
            let mut available_package_cache = self.available_packages_cache.borrow_mut();
            let available_packages = match available_package_cache.get(package.borrow()) {
                Some(packages) => packages,
                None => {
                    // Cache miss, retrieve from registry.
                    let mut available_packages = self
                        .registry
                        .all_available_versions_of_a_package(package.borrow().clone());

                    // Sort by version, descending.
                    available_packages.sort_by(|a, b| b.version.cmp(&a.version));

                    // Cache available packages for the next run.
                    available_package_cache.insert(package.borrow().clone(), available_packages);

                    #[allow(clippy::unwrap_used)]
                    // This is fine, we have just inserted it.
                    available_package_cache.get(package.borrow()).unwrap()
                }
            };

            // Pick valid versions of available packages regarding the requested version range.
            // Cache them to share the state out of this closure.
            // Return the count of valid versions.
            self.cache_and_count_valid_versions_of_package(available_packages, package, range)
        };

        let (package, range) = fallible_iterator::convert(potential_packages.map(Ok))
            .min_by_key(count_valid)?
            // Handle the error?
            // Since this piece was copied from a helper function from the pubgrub crate
            // maybe it is ok to leave it as is.
            .expect("potential_packages gave us an empty iterator");

        #[allow(clippy::unwrap_used)]
        let mut valid_versions_for_package = self
            .valid_versions_intermediate_cache
            .borrow_mut()
            .get(package.borrow())
            // This is fine, we've just filled this intermediate cache in `count_valid` closure.
            .unwrap()
            .clone();

        // Order by descending versions.
        valid_versions_for_package.sort_by(|a, b| b.cmp(a));

        let version = valid_versions_for_package.iter().copied().find(|v| {
            // Pick the first valid version, which will always be the highest version because we've sorted it.
            range.borrow().contains(v.borrow())
        });

        // We need to clear our intermediate cache in the end of this iteration to have it ready and empty for the next run.
        self.valid_versions_intermediate_cache.borrow_mut().clear();

        // Deliver the solution.
        Ok((package, version))
    }

    // Validates dependencies.
    fn get_dependencies(
        &self,
        package: &String,
        version: &SemanticVersion,
    ) -> Result<pubgrub::solver::Dependencies<String, SemanticVersion>, Box<dyn std::error::Error>>
    {
        // We need to work with our own type to query the registry.
        let package_version = (package, version).into();

        if package == ORGANIZATION_NAME {
            return Ok(pubgrub::solver::Dependencies::Known(
                // We explicitly insert and always know that we'll have the organization package in cache.
                #[allow(clippy::indexing_slicing)]
                self.available_packages_cache.borrow()[package][0]
                    .dependencies
                    .iter()
                    .try_fold(
                        DependencyConstraints::<String, SemanticVersion>::default(),
                        |mut dependencies,
                         dep|
                         -> Result<
                            DependencyConstraints<String, SemanticVersion>,
                            ResolverError,
                        > {
                            // Insert dependencies of the organization.
                            dependencies.insert(dep.full_name.clone(), dep.version_range.clone());
                            Ok(dependencies)
                        },
                    )?,
            ));
        }

        Ok(
            // TODO: In future iterations we'll add other resources for checking for dependencies such as the local file system.
            // Currently we only check registry.
            match self.registry.get_package_version_metadata(&package_version) {
                // Something went wrong and we couldn't retrieve the picked package from registry.
                Err(_) => pubgrub::solver::Dependencies::Unknown,
                // Deliver the dependencies of the picked package for this run.
                Ok(package) => {
                    let dependencies = package.dependencies.iter().try_fold(
                        DependencyConstraints::<String, SemanticVersion>::default(),
                        |mut dependencies,
                         dep|
                         -> Result<
                            DependencyConstraints<String, SemanticVersion>,
                            ResolverError,
                        > {
                            dependencies.insert(dep.full_name.clone(), dep.version_range.clone());
                            Ok(dependencies)
                        },
                    );
                    pubgrub::solver::Dependencies::Known(dependencies?)
                }
            },
        )
    }

    // An error might be returned here to terminate the operation abruptly.
    fn should_cancel(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
