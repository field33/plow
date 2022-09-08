use colored::*;
use thiserror::Error;

use crate::feedback::{command_failed, command_not_complete, linting_failed, Feedback};

#[derive(Error, Debug)]
pub enum LintSubcommandError {
    #[error("Failed to parse the field at {field_path:?}. Please make sure that you provide a valid field to lint (a valid .ttl file).")]
    FailedToParseField { field_path: String },
    #[error("Please provide a field (a valid .ttl file path) for plow to lint, {field_path:?} does not exist in the file system.")]
    FailedToFindFieldToLint { field_path: String },
    #[error("The field at {field_path:?} exists but not readable please check if the directory and file has read privileges.\n\tDetails: {details:?}")]
    FailedToReadField { field_path: String, details: String },
    #[error("Please provide a field (a valid .ttl file path) for plow to lint")]
    NoFieldProvidedToLint,
    #[error("")]
    LintsContainFailuresOpaque,
    #[error("")]
    /// This is an internal intermediate error to be used when returning early from functions
    /// which have the purpose of detecting the first error in a lint run.
    /// User feedback is not necessary.
    SingleLintContainsFailure { field_path: String },
    #[error("")]
    LintsContainFailures { field_paths: Vec<String> },
}

impl Feedback for LintSubcommandError {
    fn feedback(&self) {
        use LintSubcommandError::*;
        match self {
            FailedToParseField { .. }
            | FailedToFindFieldToLint { .. }
            | FailedToReadField { .. } => {
                command_failed(&format!("{self}"));
            }
            NoFieldProvidedToLint => {
                command_not_complete(&format!("{self}"));
            }
            LintsContainFailuresOpaque => {
                linting_failed();
            }
            SingleLintContainsFailure { .. } => { /* Omit feedback */ }
            LintsContainFailures { field_paths } => {
                println!("\t{} couldn't pass the linting phase thus ignored and not included in the workspace and fields directory.\n\tPlease lint them individually, correct them and run plow init one more time to recreate the workspace.", "Paths listed below".yellow().bold());
                for field_path in field_paths {
                    println!("\t\t{}", field_path);
                }
                println!("\t{} created successfully but partially.\n\tTo include the fields left out from the workspace, fix them and run plow init one more time to recreate the workspace.", "Workspace".yellow().bold(),);

                std::process::exit(0x0);
            }
        }
    }
}
