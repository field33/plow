use crate::feedback::{command_failed, Feedback};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceInitializationError {
    #[error("Discovering fields in directory was not successful. Details: {reason:?}")]
    FailedRecursiveListingFields { reason: String },
    #[error("Please run plow init --field @field_namespace/field_name to initialize a field or copy existing fields to this folder then run plow init to initialize the workspace.")]
    NoFieldsInDirectory,
    #[error(
        "Plow couldn't read the workspace manifest (./Plow.toml), either the file does not exist, not readable or corrupted.\n\tDetails: {0}"
    )]
    FailedToReadWorkspaceManifestFile(String),
    #[error(
        "Plow couldn't write the workspace manifest (./Plow.toml), is your workspace read only?\n\tDetails: {0}"
    )]
    FailedToWriteWorkspaceManifestFile(String),
    #[error(
        "Plow couldn't write the src directory, is your workspace read only?\n\tDetails: {0}"
    )]
    FailedToCreateFieldsDirectory(String),
    #[error(
        "Plow couldn't read the src directory, could there be permission issues?\n\tDetails: {0}"
    )]
    FailedToReadFieldsDirectory(String),
    #[error(
        "Workspace is already initialized. You may run plow init again to recreate the workspace."
    )]
    WorkspaceAlreadyInitialized,
    #[error(
        "Plow couldn't remove the workspace manifest (./Plow.toml), is your workspace read only?\n\tDetails: {0}"
    )]
    FailedToRemoveWorkspaceManifestFile(String),
    #[error(
        "Plow couldn't remove the src directory, is your workspace read only?\n\tDetails: {0}"
    )]
    FailedToRemoveFieldsDirectory(String),
    #[error(
        "Failed to remove temporary src directory. Please remove it manually.\n\tDetails: {0}"
    )]
    FailedToRemoveBackupFieldsDirectory(String),
    #[error(
        "Workspace does not support having a field with the same name multiple times. Please remove or rename the following field: {0}"
    )]
    DuplicateFieldInWorkspace(String),
}

impl Feedback for WorkspaceInitializationError {
    fn feedback(&self) {
        use WorkspaceInitializationError::*;
        match self {
            FailedRecursiveListingFields { .. }
            | NoFieldsInDirectory
            | WorkspaceAlreadyInitialized
            | FailedToCreateFieldsDirectory(_)
            | DuplicateFieldInWorkspace(_)
            | FailedToRemoveBackupFieldsDirectory(_)
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
