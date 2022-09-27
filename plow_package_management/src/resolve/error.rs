use thiserror::Error;

/// Errors related to dependency resolver.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ResolverError {
    /// An invalid version predicate has fed to resolver.
    #[error("Invalid version predicate: {0}.")]
    InvalidVersionPredicate(String),
    /// An invalid version lock file has fed to resolver.
    #[error("Invalid lock file: {0}.")]
    InvalidLockFile(String),
    /// Error related to retrieving a packages metadata from the registry.
    #[error("The package {0} was not found in registry.")]
    NotFoundInRegistry(String),
    /// Resolution error.
    #[error("{0}")]
    SolutionError(String),
}
