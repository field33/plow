use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum FieldAccessError {
    #[error("The field you've provided at {field_path:?} was not found, you may check that if the file exists.")]
    FailedToFindFieldAtPath { field_path: String },
    #[error("The field you've provided at {field_path:?} is not readable, you may check that if the file is readable in a normal text editor first.")]
    FailedToReadFieldAtPath { field_path: String },
    #[error("The manifest of the field you've provided at {field_path:?} could not be extracted. The field you have provided is probably corrupted.")]
    FailedToReadFieldManifest { field_path: String },
}

impl Feedback for FieldAccessError {
    fn feedback(&self) {
        use FieldAccessError::*;
        match self {
            FailedToFindFieldAtPath { .. }
            | FailedToReadFieldAtPath { .. }
            | FailedToReadFieldManifest { .. } => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
