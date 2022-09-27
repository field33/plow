use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[derive(Error, Debug)]
pub enum ProtegeSubcommandError {
    #[error("Please provide a field (a valid .ttl file path) for plow to open in protege.")]
    NoFieldProvidedToOpenInProtege,
    #[error("Protege workspace creation failed. Details {0}")]
    FailedToPrepareProtegeWorkspace(String),
    #[error("Running the protege executable failed. Is protege installed? Details {0}")]
    FailedToOpenProtegeApplication(String),
}

impl Feedback for ProtegeSubcommandError {
    fn feedback(&self) {
        use ProtegeSubcommandError::*;
        match self {
            NoFieldProvidedToOpenInProtege
            | FailedToPrepareProtegeWorkspace(_)
            | FailedToOpenProtegeApplication(_) => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
