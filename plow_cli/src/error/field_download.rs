use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[derive(Error, Debug)]
pub enum FieldDownloadError {
    #[error("Failed to download and cache field.\n\tDetails: {reason:?}")]
    FailedToDownloadAndCacheField { reason: String },
    #[error("Failed to read the local cache of downloaded fields.\n\tDetails: {reason:?}")]
    FailedToReadFieldCache { reason: String },
}

impl Feedback for FieldDownloadError {
    fn feedback(&self) {
        use FieldDownloadError::*;
        match self {
            FailedToDownloadAndCacheField { .. } | FailedToReadFieldCache { .. } => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
