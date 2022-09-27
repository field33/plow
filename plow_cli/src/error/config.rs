use thiserror::Error;

use crate::feedback::{command_failed, Feedback};

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("Failed to write to config directory ~/.plow.\n\tDetails: {0}")]
    FailedToWriteToConfigDirectory(String),
    #[error("Failed to write to config directory ~/.plow.\n\tDetails: {0}")]
    FailedToRemoveConfigDirectory(String),
    #[error("Failed to write to the credentials file. Make sure the directory permissions are available to write and the ~/.plow directory exists. 
    You may run `plow login <api-token>` to create a credentials file.")]
    FailedToWriteCredentialsFile,
    #[error("Failed to read or create config directory at path {0}. It is possible that the user home directory is not readable or read only. If this is the case please run plow with --config <directory-path> in your workspace root. This will create the configuration directory in the desired path and save the path into the root config.toml.\n\tDetails: {1}")]
    FailedToReadOrCreateConfigDirectory(String, String),
    #[error("Failed to retrieve the current working directory,\n\tDetails: {0}")]
    FailedToGetWorkingDirectory(String),
    #[error("This command could only be run in an initialized workspace, please either run plow init in the root of your desired workspace to create it.")]
    DirectoryNotWorkspace,
    #[error(
        "Plow couldn't create a workspace config (.plow) directory in your workspace root.\n\tDetails: {0}"
    )]
    FailedToCreateWorkspaceConfigDirectory(String),
    #[error(
        "Plow couldn't read the workspace config (.plow/config.toml), either the file does not exist, not readable or corrupted.\n\tDetails: {0}"
    )]
    FailedToReadWorkspaceConfigFile(String),
    #[error(
        "Plow couldn't write the workspace config (.plow/config.toml), is your workspace read only?\n\tDetails: {0}"
    )]
    FailedToWriteWorkspaceConfigFile(String),
    #[error(
        "Failed to find workspace root directory. Are you running the command under a directory tree which has a Plow.toml file in the root?"
    )]
    FailedToFindWorkspaceRoot,
}

impl Feedback for ConfigError {
    fn feedback(&self) {
        use ConfigError::*;
        match self {
            FailedToWriteToConfigDirectory(_)
            | FailedToRemoveConfigDirectory(_)
            | FailedToReadOrCreateConfigDirectory(_, _)
            | FailedToGetWorkingDirectory(_)
            | FailedToCreateWorkspaceConfigDirectory(_)
            | FailedToReadWorkspaceConfigFile(_)
            | FailedToWriteWorkspaceConfigFile(_)
            | FailedToWriteCredentialsFile
            | DirectoryNotWorkspace
            | FailedToFindWorkspaceRoot => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
