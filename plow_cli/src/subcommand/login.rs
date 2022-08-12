use crate::{
    config::create_configuration_directory_if_not_exists, error::CliError, feedback::Feedback,
};

use clap::{arg, App, ArgMatches, Command};
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
    token: &'reg str,
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
        .about("Registers an api token for the CLI to use.")
        .arg(arg!([API_TOKEN]))
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches) -> Box<dyn Feedback + '_> {
    match run_command_flow(sub_matches) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub fn run_command_flow(sub_matches: &clap::ArgMatches) -> Result<impl Feedback, CliError> {
    create_configuration_directory_if_not_exists()?;

    let token = sub_matches
        .get_one::<String>("API_TOKEN")
        .ok_or(NoTokenProvidedToSave)?;

    save_credentials_replace_existing(token)?;
    Ok(SuccessfulLogin)
}

pub fn save_credentials_replace_existing(token: &str) -> Result<(), CliError> {
    let credentials_contents =
        toml::to_string::<CredentialsFile>(&CredentialsFile::with_token(token))
            .map_err(|_| FailedToReadCredentialsFile)?;

    let config_dir = crate::config::get_config_dir()?;

    std::fs::write(config_dir.join("credentials.toml"), credentials_contents)
        .map_err(|_| FailedToWriteCredentialsFile)?;
    Ok(())
}

pub fn get_saved_api_token() -> Result<String, CliError> {
    let config_dir = crate::config::get_config_dir().map_err(|_| FailedToReadCredentialsFile)?;
    let credentials_file_contents = std::fs::read_to_string(config_dir.join("credentials.toml"))
        .map_err(|_| FailedToReadCredentialsFile)?;
    let credentials = toml::from_str::<CredentialsFile>(&credentials_file_contents)
        .map_err(|_| FailedToReadCredentialsFile)?;
    Ok(credentials.registry.token.to_owned())
}
