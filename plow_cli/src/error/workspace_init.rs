use crate::feedback::{command_failed, Feedback};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceInitializationError {
    #[error("Discovering fields in directory was not successful. Details: {reason:?}")]
    FailedRecursiveListingFields { reason: String },
    #[error("Please run this command in a directory containing fields (.ttl files) in any depth.")]
    NoFieldsInDirectory,
    #[error(
        "Plow couldn't read the workspace manifest (./Plow.toml), either the file does not exist, not readable or corrupted. Details: {0}"
    )]
    FailedToReadWorkspaceManifestFile(String),
    #[error(
        "Plow couldn't write the workspace manifest (./Plow.toml), is your workspace read only? Details: {0}"
    )]
    FailedToWriteWorkspaceManifestFile(String),
    #[error(
        "Plow couldn't write the fields directory (fields), is your workspace read only? Details: {0}"
    )]
    FailedToCreateFieldsDirectory(String),
    #[error(
        "Plow couldn't read the fields directory (fields), could there be permission issues? Details: {0}"
    )]
    FailedToReadFieldsDirectory(String),
    #[error(
        "Workspace is already initialized. You may run plow init --force to recreate the workspace."
    )]
    WorkspaceAlreadyInitialized,
    #[error(
        "Plow couldn't remove the workspace manifest (./Plow.toml), is your workspace read only? Details: {0}"
    )]
    FailedToRemoveWorkspaceManifestFile(String),
    #[error(
        "Plow couldn't remove the fields directory (fields), is your workspace read only? Details: {0}"
    )]
    FailedToRemoveFieldsDirectory(String),
}

impl Feedback for WorkspaceInitializationError {
    fn feedback(&self) {
        use WorkspaceInitializationError::*;
        match self {
            FailedRecursiveListingFields { .. }
            | NoFieldsInDirectory
            | WorkspaceAlreadyInitialized
            | FailedToCreateFieldsDirectory(_)
            | FailedToReadFieldsDirectory(_)
            | FailedToReadWorkspaceManifestFile(_)
            | FailedToRemoveFieldsDirectory(_)
            | FailedToRemoveWorkspaceManifestFile(_)
            | FailedToWriteWorkspaceManifestFile(_) => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
