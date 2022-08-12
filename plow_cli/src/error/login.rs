use thiserror::Error;

use crate::feedback::{command_not_complete, login_failed, Feedback};

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Failed to read the credentials file. Make sure the file exists and is readable. You may run, plow login <api-token> to create a credentials file.")]
    FailedToReadCredentialsFile,
    #[error("Failed to write to the credentials file. Make sure the directory permissions are available to write and the ~/.plow directory exists. 
    You may run `plow login <api-token>` to create a credentials file.")]
    FailedToWriteCredentialsFile,
    #[error("Please provide a valid API token to save")]
    NoTokenProvidedToSave,
}

impl Feedback for LoginError {
    fn feedback(&self) {
        use LoginError::*;
        match self {
            FailedToReadCredentialsFile | FailedToWriteCredentialsFile => {
                login_failed(&format!("{self}"));
            }
            NoTokenProvidedToSave => {
                command_not_complete(&format!("{self}"));
            }
        }
    }
}
