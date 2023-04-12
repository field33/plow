mod response;

use crate::config::PlowConfig;
use crate::error::CliError;
use crate::error::FieldAccessError::*;
use crate::error::ListError::*;

use crate::feedback::*;
use crate::manifest::FieldManifest;
use crate::resolve::resolve;
// use anyhow::Ok;
// use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{arg, App, AppSettings, Arg, ArgMatches, Command};
use colored::Colorize;
use plow_linter::lints::field_manifest_lints;
use plow_package_management::registry::Registry;
use reqwest::blocking::multipart::Form;

use self::response::{RegistryResponse, StatusInfo};
use super::lint::lint_file;

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("list")
        .about("This command is the entry point for retrieving information about the plow user and the ecosystem.")
        .arg(
            Arg::with_name("organizations")
                .long("organizations")
                .help("List the organizations which a plow user is a member of if there is any."),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches, config: &PlowConfig) -> Box<dyn Feedback + 'static> {
    match run_command_flow(sub_matches, config) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

fn run_command_flow(
    sub_matches: &ArgMatches,
    config: &PlowConfig,
) -> Result<impl Feedback, CliError> {
    let list_organizations = sub_matches.is_present("organizations");

    if list_organizations {
        let response = send_listing_request(config)?;
        return Ok(response);
    }
    Err(CliError::from(ArgumentNotRecognized))
}

fn send_listing_request(config: &PlowConfig) -> Result<RegistryResponse, CliError> {
    let token = config.get_saved_api_token()?;
    let registry_url = config.get_registry_url()?;
    let list_orgs_url = format!("{registry_url}/v1/user/organizations");

    let client = reqwest::blocking::Client::new();
    let listing_response = client
        .get(list_orgs_url)
        .header(reqwest::header::AUTHORIZATION, &format!("Basic {token}"))
        .send()
        .map_err(|err| RequestFailed { code: err.status() })?;

    let status = listing_response.status();
    let response_body_value =
        listing_response
            .json::<serde_json::Value>()
            .map_err(|_| NoValidBodyInResponse {
                registry_url: registry_url.to_owned(),
            })?;

    let response_jsend_status_text = response_body_value
        .get("status")
        .ok_or(InvalidResponseFromRegistry { code: status })?
        .as_str()
        .ok_or(InvalidResponseFromRegistry { code: status })?;
    let response_body_contents = response_body_value.to_string();
    let status_info = StatusInfo::try_from(response_jsend_status_text)
        .map_err(|_| InvalidResponseFromRegistry { code: status })?;

    match status_info {
        StatusInfo::Error => {
            let response: RegistryResponse =
                serde_json::from_str::<self::response::Error>(&response_body_contents)
                    .map_err(|_| InvalidResponseFromRegistry { code: status })?
                    .into();
            Ok(response)
        }
        StatusInfo::Failure => Ok(serde_json::from_str::<self::response::Failure>(
            &response_body_contents,
        )
        .map_err(|_| InvalidResponseFromRegistry { code: status })?
        .into()),
        StatusInfo::Success => {
            let contents = serde_json::from_str::<self::response::Success>(&response_body_contents)
                .map_err(|_| InvalidResponseFromRegistry { code: status })?;
            let data = contents.data;
            match data {
                self::response::Data::UserOrganizations { organizations } => {
                    Ok(RegistryResponse::ListingSuccess { organizations })
                }
                _ => Err(InvalidResponseFromRegistry { code: status }.into()),
            }
        }
    }
}
