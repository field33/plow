mod response;

use crate::config::PlowConfig;
use crate::error::CliError;
use crate::error::FieldAccessError::*;
use crate::error::SubmissionError::*;

use crate::feedback::*;
use crate::manifest::FieldManifest;
use crate::resolve::resolve;
use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{arg, App, AppSettings, Arg, ArgMatches, Command};
use colored::Colorize;
use plow_linter::lints::field_manifest_lints;
use plow_package_management::registry::Registry;
use reqwest::blocking::multipart::Form;

use self::response::{RegistryResponse, StatusInfo};
use super::lint::lint_file;

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("submit")
        .about("Submits a field to the specified registry.")
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .help("Will go through all operations of a submission locally and in remote but not persist the results.")
        )
        .arg(
            Arg::with_name("private")
                .long("private")
                .help("Submit the field privately.")
        )
        .arg(arg!([FIELD_PATH]))
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
    let field_file_path_arg = sub_matches
        .get_one::<String>("FIELD_PATH")
        .ok_or(FieldPathNotProvided)?;
    let field_file_path = camino::Utf8PathBuf::from(field_file_path_arg);

    if field_file_path.exists() {
        field_info(&field_file_path)?;

        lint_file(field_file_path.as_str(), vec![field_manifest_lints()])
            .map_err(|_| LintingFailed)?;

        general_lint_success();

        let path = Utf8PathBuf::from(&field_file_path);

        let registry = crate::sync::sync(config).map_err(|err| CliError::Wip(err.to_string()))?;

        let root_field_contents = std::fs::read_to_string(&path).map_err(|_| {
            CliError::from(FailedToFindFieldAtPath {
                field_path: path.to_string(),
            })
        })?;
        let root_field_manifest =
            FieldManifest::new(root_field_contents.clone()).map_err(|_| {
                CliError::from(FailedToReadFieldManifest {
                    field_path: path.to_string(),
                })
            })?;

        if let Some(lock_file) = resolve(
            config,
            &root_field_contents,
            &root_field_manifest,
            true,
            &registry as &dyn Registry,
        )? {
            // Leave an empty line in between.
            println!();
            println!("\t{}", "Dependencies".bold().green());
            lock_file
                .locked_dependencies
                .packages
                .iter()
                .for_each(|package_version| {
                    println!(
                        "\t\t{} {}",
                        package_version.package_name.bold(),
                        package_version.version
                    );
                });
        }

        // File linted and ready to submit.
        let public = !sub_matches.is_present("private");
        let dry_run = sub_matches.is_present("dry-run");

        let submission = reqwest::blocking::multipart::Form::new()
            .text("public", if public { "true" } else { "false" })
            .file("field", &field_file_path)
            .map_err(|_| FailedToReadFieldAtPath {
                field_path: field_file_path.clone().into(),
            })?;

        // Read credentials
        let token = config.get_saved_api_token()?;
        let registry_url = config.get_registry_url()?;

        let mut submission_url = format!("{registry_url}/v1/field/submit");
        if dry_run {
            submission_url.push_str("?dry-run=true");
        }

        return send_submission(&submission_url, &registry_url, submission, &token, dry_run);
    }
    Err(FailedToFindFieldAtPath {
        field_path: field_file_path.into(),
    }
    .into())
}

fn send_submission(
    submission_url: &str,
    registry_url: &str,
    submission: Form,
    token: &str,
    dry_run: bool,
) -> Result<RegistryResponse, CliError> {
    let client = reqwest::blocking::Client::new();

    let submission_response = client
        .post(submission_url)
        .header("Authorization", &format!("Basic {token}"))
        .multipart(submission)
        .send()
        .map_err(|err| RequestFailed { code: err.status() })?;

    let status = submission_response.status();
    let response_body_value = submission_response
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
            let response = serde_json::from_str::<self::response::Error>(&response_body_contents)
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
            serde_json::from_str::<self::response::Success>(&response_body_contents)
                .map_err(|_| InvalidResponseFromRegistry { code: status })?;
            Ok(RegistryResponse::SubmissionSuccess { dry_run })
        }
    }
}
