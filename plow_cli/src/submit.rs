mod response;

use crate::{
    config::get_registry_url, feedback::*, lint::lint_file, login::get_saved_api_token,
    submit::response::StatusInfo,
};
use anyhow::Result;
use clap::{arg, App, Arg, ArgMatches, Command};
use colored::*;
use plow_linter::lints::required_plow_registry_lints;
use reqwest::blocking::multipart::Form;

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("submit")
        .about("Submits an ontology to plow registry.")
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
}

pub fn run_command(sub_matches: &ArgMatches) -> Result<()> {
    if let Some(field_file_path) = sub_matches.get_one::<String>("FIELD_PATH") {
        let field_file_path = camino::Utf8PathBuf::from(field_file_path);
        if field_file_path.exists() {
            println!(
                "\t{} the field before submission..",
                "Linting".green().bold(),
            );
            // Ready to do pre submission lint.
            if lint_file(
                field_file_path.as_str(),
                Some(required_plow_registry_lints()),
            )
            .is_err()
            {
                linting_failed();
            }
            println!("\t{} successful.", "Linting".green().bold(),);

            // File linted and ready to submit.
            let public = !sub_matches.is_present("private");
            let dry_run = sub_matches.is_present("dry-run");

            let registry_url = if sub_matches.is_present("registry") {
                let registry_url = sub_matches.get_one::<String>("registry");
                registry_url.map_or_else(
                    || {
                        command_not_complete("Try providing a valid registry url.");
                        // Dummy
                        "".to_owned()
                    },
                    std::borrow::ToOwned::to_owned,
                )
            } else if let Ok(url) = get_registry_url() {
                url
            } else {
                command_not_complete("Try providing a valid registry url either in Plow.toml or with a command line argument.");
                // Dummy
                "".to_owned()
            };

            // TODO: Handle these errors properly.
            let submission = reqwest::blocking::multipart::Form::new()
                .text("public", if public { "true" } else { "false" })
                .file("field", field_file_path)?;

            // Read auth.
            let token = get_saved_api_token().unwrap_or_else(|err| {
            command_failed(&format!(
                "Could not read API token from ~/.plow/credentials.toml: {}. Please run `plow login <api-token>` to login.",
                err
            ));
            // Dummy
            "".to_owned()
        });

            let mut submission_url = format!("{registry_url}/v1/field/submit");
            if dry_run {
                submission_url.push_str("?dry-run=true");
            }

            return send_submission(&submission_url, &registry_url, submission, &token, dry_run);
        }
        command_failed(
                &format!("The field you've provided at {field_file_path} is not readable, you may check that if the file is readable in a normal text editor first."),
            );
    }
    Ok(())
}

pub fn send_submission(
    submission_url: &str,
    registry_url: &str,
    submission: Form,
    token: &str,
    dry_run: bool,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let submission_response = client
        .post(submission_url)
        .header("Authorization", &format!("Basic {token}"))
        .multipart(submission)
        .send()?;
    let status = submission_response.status();
    if let Ok(response_body_value) = submission_response.json::<serde_json::Value>() {
        response_body_value.get("status").map_or_else(|| {
                submission_failed("Registry sent an invalid response.");
            }, |status_text_value| {
                let response_body_contents = response_body_value.to_string();
                status_text_value.as_str().map_or_else(|| {
                    submission_failed("Registry sent an invalid response which doesn't conform to jsend standard.");
                }, |status_text| if let Ok(status_info) = StatusInfo::try_from(status_text) {
                        // Do something with the responses..
                        match status_info {
                            StatusInfo::Error => {
                                if let Ok(response) = serde_json::from_str::<
                                    crate::submit::response::Error,
                                >(
                                    &response_body_contents
                                ) {
                                    submission_failed(response.message);
                                } else {
                                    submission_failed(
                                        "Registry sent an invalid response.",
                                    );
                                }
                            }
                            StatusInfo::Failure => {
                                if let Ok(response) = serde_json::from_str::<
                                    crate::submit::response::Failure,
                                >(
                                    &response_body_contents
                                ) {
                                    match response.data {
                                        crate::submit::response::Data::FailureMessage(reason) => {
                                            submission_failed(reason);
                                        },
                                        crate::submit::response::Data::SubmissionLintingResults {
                                            failures
                                        } => {
                                            submission_remote_linting_failed(&failures);
                                        }
                                    }
                                } else {
                                    submission_failed(
                                        "Registry sent an invalid response.",
                                    );
                                }
                            }
                            StatusInfo::Success => {
                                if serde_json::from_str::<
                                    crate::submit::response::Success,
                                >(
                                    &response_body_contents
                                ).is_ok() {
                                    if dry_run {
                                        println!(
                                        "\t{} run was successful. You may safely submit the field to plow.",
                                        "Submission".green().bold(),
                                        );
                                    }
                                    else {
                                        println!(
                                        "\t{} successful. The field is now uploaded to plow.",
                                        "Submission".green().bold(),
                                        );
                                    }
                                } else {
                                    submission_failed(
                                        "Registry sent an invalid response.",
                                    );
                                }
                            }
                        };
                    } else {
                        submission_failed("Registry sent an invalid response which doesn't conform to jsend standard.");
                    });
            });
    } else {
        command_failed(&format!("Submission failed with status code {status}. There was no valid body in the response. You may check weather the registry url you have provided is right. Your current registry url is {registry_url}"));
    }
    Ok(())
}
