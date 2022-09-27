use crate::feedback::{command_failed, Feedback};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum IndexSyncError {
    #[error("Index is corrupted.\n\tDetails: {0}")]
    FailedToParseIndex(String),
    #[error("Failed to read index directory.\n\tDetails: {0}")]
    FailedToReadIndexDirectory(String),
    #[error("Failed to retrieve public index repository.\n\tDetails: {0}")]
    FailedToGetRepository(String),
    #[error("Private index sync failed.\n\tDetails: {0}")]
    FailedToGetPrivateIndexUpdates(String),
}

impl Feedback for IndexSyncError {
    fn feedback(&self) {
        use IndexSyncError::*;
        match self {
            FailedToParseIndex(_)
            | FailedToReadIndexDirectory(_)
            | FailedToGetRepository(_)
            | FailedToGetPrivateIndexUpdates(_) => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
