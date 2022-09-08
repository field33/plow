use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[derive(Error, Debug)]
pub enum FieldInitializationError {
    #[error("The provided field name is invalid.\n\tDetails: {reason:?}")]
    InvalidFieldNameProvided { reason: String },
    #[error("Please provide a valid field name to create. e.g. @my_fields/precious_field")]
    NoFieldNameProvided,
    #[error("Failed to write field to the fields directory.\n\tDetails: {0}")]
    FailedToWriteField(String),
}

impl Feedback for FieldInitializationError {
    fn feedback(&self) {
        use FieldInitializationError::*;
        match self {
            InvalidFieldNameProvided { .. } | NoFieldNameProvided | FailedToWriteField(_) => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
