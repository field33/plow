use anyhow::bail;
use clap::{ArgMatches, App, Arg, Command, arg};
use plow_linter::lints::required_plow_registry_lints;
use crate::{feedback::*, lint::lint_file, config::get_registry_url, login::get_saved_api_token, submit::response::StatusInfo};
use anyhow::Result;
use colored::*;

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
        let lock_file_path: Option<camino::Utf8PathBuf> = camino::Utf8PathBuf::from(&field_file_path).parent().map_or_else(|| {
            info(
                "Could not find a lock file in the same directory with the field, creating a lock file with this command not implemented yet. If you place the lock file in the same folder with the field submission would work normally.\n On the other hand you might have no dependencies at all and you can just submit the field directly. If you have dependencies stated in the field but not provide a lock file your submission will fail though.");
            // TODO: Try Resolve deps and create a lock file and return contents later change info message and then allow submission without a lock file.
            None
        }, |parent| {
            // TODO: Update lock file name later..
            let possible_lock_file_path = parent.join("Ontology.lock");

            let lock_file_path = if possible_lock_file_path.exists() {
                Some(possible_lock_file_path)
            } else {
                info(
                &format!("Could not find a lock file at {possible_lock_file_path}, creating a lock file with this command not implemented yet. If you place the lock file in the same folder with the field submission would work normally.\n On the other hand you might have no dependencies at all and you can just submit the field directly. If you have dependencies stated in the field but not provide a lock file your submission will fail though."));

                // TODO: Try Resolve deps and create a lock file and return contents later change info message and then allow submission without a lock file.
                None
            };
            lock_file_path
        });

        // Ready to do pre submission lint.
        if lint_file(
            field_file_path.as_str(),
            Some(required_plow_registry_lints()),
        )
        .is_err()
        {
            command_failed(
                "Depending on the red lines in the output, try to fix your field and try again.",
            );
        }

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
        let submission = if let Some(lock_file_path) = lock_file_path {
            reqwest::blocking::multipart::Form::new()
                .text("public", if public { "true" } else { "false" })
                .file("field", field_file_path)?
                .file("lock_file", lock_file_path)?
        } else {
            reqwest::blocking::multipart::Form::new()
                .text("public", if public { "true" } else { "false" })
                .file("field", field_file_path)?
        };

        let client = reqwest::blocking::Client::new();

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
            // TODO: Should I url encode this?
            // Or reqwest does it?
            submission_url.push_str("?dry-run=true");
        }

        let submission_response = client
            .post(&submission_url)
            .header("Authorization", &format!("Basic {token}"))
            .multipart(submission)
            .send()?;
        let status = submission_response.status();

        if let Ok(response_body_value) = submission_response.json::<serde_json::Value>()
        {
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
                                    println!(
                                        "\t{} successful. The field is now uploaded to plow.",
                                        "Submission".green().bold(),
                                    );                                     
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
        return Ok(());
    }
    command_failed(
                &format!("The field you've provided at {field_file_path} is not readable, you may check that if the file is readable in a normal text editor first."),
            );
    }
    bail!("Please give a file path to a ttl file to lint.");
}


pub mod response {
    use anyhow::bail;
    use serde::{Deserialize, Serialize};

    /// `status` field of the response.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub enum StatusInfo {
        #[serde(rename(serialize = "success", deserialize = "success"))]
        Success,
        #[serde(rename(serialize = "fail", deserialize = "fail"))]
        Failure,
        #[serde(rename(serialize = "error", deserialize = "error"))]
        Error,
    }

    impl TryFrom<&str> for StatusInfo {
        type Error = anyhow::Error;
        fn try_from(s: &str) -> Result<Self, anyhow::Error> {
            match s {
                "success" => Ok(StatusInfo::Success),
                "fail" => Ok(StatusInfo::Failure),
                "error" => Ok(StatusInfo::Error),
                s => bail!("Invalid status text: {}", s),
            }
        }
    }
    /// A response with success status.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend#success) spec
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Success<'body> {
        pub status: StatusInfo,
        #[serde(borrow)]
        pub data: Option<Data<'body>>,
    }

    /// A response with fail status.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend#fail) spec
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Failure<'body> {
        pub status: StatusInfo,
        #[serde(borrow)]
        pub data: Data<'body>,
    }

    /// A response with error status.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend#error) spec
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Error<'body> {
        pub status: StatusInfo,
        pub code: u16,
        pub error: &'body str,
        pub message: &'body str,
        pub data: Option<Data<'body>>,
        pub timestamp: String,
    }

    #[allow(clippy::large_enum_variant)]
    /// `data` field of the response.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Data<'data> {
        FailureMessage(&'data str),
        // Serialized as {"field": "...", }
        SubmissionLintingResults {
            /// Non exhaustive list of linting failure messages.
            failures: Vec<String>,
        },
    }
}