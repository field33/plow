use crate::{config::PlowConfig, error::CliError, feedback::Feedback};

use clap::{arg, App, AppSettings, ArgMatches, Command};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::error::LoginError::*;

#[derive(Serialize, Debug, Deserialize)]
pub struct CredentialsFile<'cred> {
    #[serde(borrow)]
    pub registry: Registry<'cred>,
}

impl<'cred> CredentialsFile<'cred> {
    /// Returns the token for the registry.
    pub const fn with_token(token: &'cred str) -> Self {
        CredentialsFile {
            registry: Registry::new(token),
        }
    }
}

/// Registry table in credentials file (toml).
#[derive(Serialize, Debug, Deserialize)]
pub struct Registry<'reg> {
    pub token: &'reg str,
}

impl<'reg> Registry<'reg> {
    /// Returns the token for the registry.
    pub const fn new(token: &'reg str) -> Self {
        Registry { token }
    }
}

pub struct SuccessfulLogin;
impl Feedback for SuccessfulLogin {
    fn feedback(&self) {
        println!(
            "\t{} successful. Saved API token to ~/.plow/credentials.toml",
            "Login".green().bold(),
        );
    }
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("login")
        .about("Registers an api token to interact with remote registries.")
        .arg(arg!([API_TOKEN]))
        .setting(AppSettings::ArgRequiredElseHelp)
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches, config: &PlowConfig) -> Box<dyn Feedback + 'static> {
    match run_command_flow(sub_matches, config) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub fn run_command_flow(
    sub_matches: &clap::ArgMatches,
    config: &PlowConfig,
) -> Result<impl Feedback, CliError> {
    let token = sub_matches
        .get_one::<String>("API_TOKEN")
        .ok_or(NoTokenProvidedToSave)?;

    save_credentials_replace_existing(token, config)?;
    Ok(SuccessfulLogin)
}

pub fn save_credentials_replace_existing(token: &str, config: &PlowConfig) -> Result<(), CliError> {
    let credentials_contents =
        toml::to_string::<CredentialsFile>(&CredentialsFile::with_token(token))
            .map_err(|_| FailedToReadCredentialsFile)?;

    std::fs::write(&config.credentials_path, credentials_contents)
        .map_err(|_| FailedToWriteCredentialsFile)?;
    Ok(())
}
