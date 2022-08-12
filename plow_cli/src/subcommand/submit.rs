mod response;

use crate::error::SubmissionError::*;
use crate::error::{CliError, SubmissionError};
use crate::{config::get_registry_url, feedback::*};
use anyhow::Result;
use clap::{arg, App, AppSettings, Arg, ArgMatches, Command};
use plow_linter::lints::required_plow_registry_lints;
use reqwest::blocking::multipart::Form;

use self::response::{RegistryResponse, StatusInfo};

use super::lint::lint_file;
use super::login::get_saved_api_token;

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("submit")
        .about("Submits a field to the specified registry.")
        .arg(
            Arg::with_name("registry")
                .short('r')
                .value_name("REGISTRY_PATH")
                .long("registry")
                .help("Specifies the target registry to submit.")
                .takes_value(true),
        )
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
pub fn run_command(sub_matches: &ArgMatches) -> Box<dyn Feedback + '_> {
    match run_command_flow(sub_matches) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

fn run_command_flow(sub_matches: &ArgMatches) -> Result<impl Feedback, CliError> {
    let field_file_path_arg = sub_matches
        .get_one::<String>("FIELD_PATH")
        .ok_or(FieldPathNotProvided)?;
    let field_file_path = camino::Utf8PathBuf::from(field_file_path_arg);

    if field_file_path.exists() {
        submission_lint_start();

        lint_file(
            field_file_path.as_str(),
            Some(required_plow_registry_lints()),
        )
        .map_err(|_| LintingFailed)?;

        submission_lint_success();

        // File linted and ready to submit.
        let public = !sub_matches.is_present("private");
        let dry_run = sub_matches.is_present("dry-run");

        let registry_url = if sub_matches.is_present("registry") {
            sub_matches
                .get_one::<String>("registry")
                .ok_or(RegistryPathNotProvided)?
                .clone()
        } else {
            get_registry_url()?
        };

        let submission = reqwest::blocking::multipart::Form::new()
            .text("public", if public { "true" } else { "false" })
            .file("field", &field_file_path)
            .map_err(|_| SubmissionError::FailedToReadFieldAtPath {
                field_path: field_file_path.into(),
            })?;

        // Read credentials
        let token = get_saved_api_token()?;

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
