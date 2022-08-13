use crate::feedback::{command_failed, Feedback};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceInitializationError {
    #[error("Discovering fields in directory was not successful. Details: {reason:?}")]
    FailedRecursiveListingFields { reason: String },
    #[error("Please run this command in a directory containing fields (.ttl files) in any depth.")]
    NoFieldsInDirectory,
    #[error("Failed to create fields directory in current directory. Details: {0}")]
    FailedToCreateFieldsDirectory(String),
    #[error("Failed to write to fields directory. Details: {0}")]
    FailedToWriteToFieldsDirectory(String),
    #[error("Failed to create Plow.toml in the workspace. Details: {0}")]
    FailedToCreatePlowToml(String),
}

impl Feedback for WorkspaceInitializationError {
    fn feedback(&self) {
        use WorkspaceInitializationError::*;
        match self {
            FailedRecursiveListingFields { .. }
            | NoFieldsInDirectory
            | FailedToCreateFieldsDirectory(_)
            | FailedToWriteToFieldsDirectory(_)
            | FailedToCreatePlowToml(_) => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
