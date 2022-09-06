#![allow(clippy::pub_use)]

mod config;
mod field_access;
mod field_download;
mod field_init;
mod index_sync;
mod lint;
mod login;
mod resolve;
mod submission;
mod workspace_init;

pub use config::ConfigError;
pub use field_access::FieldAccessError;
pub use field_download::FieldDownloadError;
pub use field_init::FieldInitializationError;
pub use index_sync::IndexSyncError;
pub use lint::LintSubcommandError;
pub use login::LoginError;
pub use resolve::ResolveError;
pub use submission::SubmissionError;
pub use workspace_init::WorkspaceInitializationError;

use crate::feedback::{command_failed, command_not_complete, Feedback};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("")]
    Submission(SubmissionError),
    #[error("")]
    Login(LoginError),
    #[error("")]
    Config(ConfigError),
    #[error("")]
    LintSubcommand(LintSubcommandError),
    #[error("")]
    FieldInitialization(FieldInitializationError),
    #[error("")]
    FieldDownload(FieldDownloadError),
    #[error("")]
    WorkspaceInitialization(WorkspaceInitializationError),
    #[error("")]
    IndexSync(IndexSyncError),
    #[error("")]
    Resolve(ResolveError),
    #[error("")]
    FieldAccess(FieldAccessError),
    #[error("The command line option you have provided is not in the list of options. Please run plow --help to see the list of options.")]
    UnknownOption,
    #[error("Do not use when publishing, intended for fast development.")]
    Dummy,
    #[error("The Plow CLI is in alpha state and a work in progress. Probably we didn't yet enough time to handle this error. Please update frequently it is most likely to be handled soon.\n\tDetails: {0}")]
    Wip(String),
}

impl From<SubmissionError> for CliError {
    fn from(error: SubmissionError) -> Self {
        Self::Submission(error)
    }
}

impl From<LoginError> for CliError {
    fn from(error: LoginError) -> Self {
        Self::Login(error)
    }
}

impl From<ConfigError> for CliError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<LintSubcommandError> for CliError {
    fn from(error: LintSubcommandError) -> Self {
        Self::LintSubcommand(error)
    }
}

impl From<FieldInitializationError> for CliError {
    fn from(error: FieldInitializationError) -> Self {
        Self::FieldInitialization(error)
    }
}

impl From<WorkspaceInitializationError> for CliError {
    fn from(error: WorkspaceInitializationError) -> Self {
        Self::WorkspaceInitialization(error)
    }
}
impl From<FieldAccessError> for CliError {
    fn from(error: FieldAccessError) -> Self {
        Self::FieldAccess(error)
    }
}

impl From<FieldDownloadError> for CliError {
    fn from(error: FieldDownloadError) -> Self {
        Self::FieldDownload(error)
    }
}

impl From<IndexSyncError> for CliError {
    fn from(error: IndexSyncError) -> Self {
        Self::IndexSync(error)
    }
}

impl From<ResolveError> for CliError {
    fn from(error: ResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl Feedback for CliError {
    fn feedback(&self) {
        use CliError::*;
        match self {
            Submission(error) => error.feedback(),
            Login(error) => error.feedback(),
            Config(error) => error.feedback(),
            LintSubcommand(error) => error.feedback(),
            FieldInitialization(error) => error.feedback(),
            FieldDownload(error) => error.feedback(),
            IndexSync(error) => error.feedback(),
            Resolve(error) => error.feedback(),
            WorkspaceInitialization(error) => error.feedback(),
            FieldAccess(error) => error.feedback(),
            UnknownOption => {
                command_not_complete(&format!("{self}"));
            }
            Dummy | Wip(_) => {
                command_failed(&format!("{self}"));
            }
        }
    }
}
