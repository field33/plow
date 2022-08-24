use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[derive(Error, Debug)]
pub enum FieldInitializationError {
    #[error("The provided field name is invalid. Details: {reason:?}")]
    InvalidFieldNameProvided { reason: String },
    #[error("Please provide a valid field name to create. e.g. @my_fields/precious_field")]
    NoFieldNameProvided,
}

impl Feedback for FieldInitializationError {
    fn feedback(&self) {
        use FieldInitializationError::*;
        match self {
            InvalidFieldNameProvided { .. } | NoFieldNameProvided => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
