use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("Failed to find config directory ~/.plow. Details: {0}")]
    FailedToGetConfigDirectory(String),
    #[error("Failed to write to config directory ~/.plow. Details: {0}")]
    FailedToWriteToConfigDirectory(String),
    #[error("Failed to write to config directory ~/.plow. Details: {0}")]
    FailedToRemoveConfigDirectory(String),
    #[error("Failed to read ./Plow.toml, please make sure the file exists and is readable.")]
    FailedToReadWorkspaceConfigFile,
    #[error("Failed to write to the credentials file. Make sure the directory permissions are available to write and the ~/.plow directory exists. 
    You may run `plow login <api-token>` to create a credentials file.")]
    FailedToWriteCredentialsFile,
}

impl Feedback for ConfigError {
    fn feedback(&self) {
        use ConfigError::*;
        match self {
            FailedToGetConfigDirectory(_)
            | FailedToWriteToConfigDirectory(_)
            | FailedToRemoveConfigDirectory(_)
            | FailedToReadWorkspaceConfigFile
            | FailedToWriteCredentialsFile => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
