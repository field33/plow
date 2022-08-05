#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,
    clippy::non_ascii_literal,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit,
    // We decided that we're ok with expect
    clippy::expect_used,
    clippy::wildcard_imports,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,
    // Currently breaks CI, let's wait a bit more until new clippy version is more spread.
    // clippy::single_char_lifetime_names,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    clippy::wildcard_dependencies,
    clippy::redundant_feature_names,
    clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,

    clippy::negative_feature_names
)]

///! A premature version of a future cli for `ontology_tools`.
///! It only collects ideas now and will be architected and re-written in the future.
pub mod config;
mod login;
mod submit;
mod util;

use anyhow::{anyhow, bail, Result};
use clap::{arg, App, Arg, Command};
use config::{create_configuration_directory_if_not_exists, PlowConfigFile};
use dialoguer::console::Emoji;
use dialoguer::Input;
use harriet::TurtleDocument;
use nom::error::VerboseError;
use plow_linter::lint::{Lint, LintResult};
use plow_ontology::{initialize_ontology, validate_ontology_name};
// This is currently fine for this stage of this binary.
#[allow(clippy::wildcard_imports)]
use plow_linter::lints::*;
use plow_package_management::{
    lock,
    package::{FieldMetadata, OrganizationToResolveFor},
};
use reqwest::Client;
use util::get_a_list_of_requested_dependencies_from_a_field;

use crate::{
    login::{get_saved_api_token, save_credentials_replace_existing},
    submit::response::StatusInfo,
};
use colored::*;

pub static SUCCESS: Emoji = Emoji("✅  ", "SUCCESS");
pub static WARNING: Emoji = Emoji("⚠️  ", "MAYBE");
pub static FAILURE: Emoji = Emoji("❌  ", "FAILURE");

#[allow(clippy::missing_panics_doc)]
#[allow(clippy::too_many_lines)]
// TODO: Make a scope and a development plan for this project, it is currently just a sketch.
pub fn main() -> Result<()> {
    let matches = App::new("plow")
        .version("0.1.0")
        .author("Maximilian Goisser <max@field33.com>, Ali Somay <ali@field33.com>")
        .about("A command line application to apply certain operations to ontologies.")
        .arg(
            Arg::with_name("init")
                .long("init")
                .help("Initializes an ontology."),
        )
        .subcommand(
            Command::new("lint")
                .about("Lints a field.")
                .arg(arg!([FIELD_PATH])),
        )
        .subcommand(
            Command::new("login")
                .about("Registers an api token for the CLI to use.")
                .arg(arg!([API_TOKEN])),
        )
        .subcommand(
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
                .arg(arg!([FIELD_PATH])),
        )
        .get_matches();

    if matches.is_present("init") {
        initialize()?;
    }

    match matches.subcommand() {
        Some(("login", sub_matches)) => {
            create_configuration_directory_if_not_exists()?;
            if let Some(token) = sub_matches.get_one::<String>("API_TOKEN") {
                save_credentials_replace_existing(token)?;
                println!(
                    "\t{} successful. Saved API token to ~/.plow/credentials.toml",
                    "Login".green().bold(),
                );
                return Ok(());
            }
            command_not_complete("please provide a valid API token to save");
        }
        Some(("lint", sub_matches)) => {
            if let Some(file_path) = sub_matches.get_one::<String>("FIELD_PATH") {
                let path_buf = camino::Utf8PathBuf::from(file_path);
                if path_buf.exists() {
                    // TODO: Add specific lints here.
                    lint_file(file_path, None)?;

                    println!("\t{} successful.", "Lint".green().bold(),);
                    return Ok(());
                }
                command_not_complete(
                    &format!("please provide a field (a valid .ttl file path) for plow to lint, {file_path} does not exist"),
                );
            }
            command_not_complete(
                "please provide a field (a valid .ttl file path) for plow to lint",
            );
        }
        Some(("submit", sub_matches)) => {
            dbg!(&sub_matches);
            if let Some(field_file_path) = sub_matches.get_one::<String>("FIELD_PATH") {
                let field_file_path = camino::Utf8PathBuf::from(field_file_path);
                if field_file_path.exists() {
                    let lock_file_path: camino::Utf8PathBuf = if let Some(parent) =
                        camino::Utf8PathBuf::from(&field_file_path).parent()
                    {
                        // TODO: Update lock file name later..
                        let possible_lock_file_path = parent.join("Ontology.lock");

                        let lock_file_path = if possible_lock_file_path.exists() {
                            possible_lock_file_path
                        } else {
                            command_failed(
                            &format!("Could not find a lock file at {possible_lock_file_path}, creating a lock file with this command not implemented yet. If you place the lock file in the same folder with the field submission would work normally."));

                            // TODO: Resolve deps and create a lock file and return contents later.
                            todo!()
                        };
                        lock_file_path
                    } else {
                        command_failed(
                            &format!("Could not find a lock file in the same folder of the field, creating a lock file with this command not implemented yet. If you place the lock file in the same folder with the field submission would work normally."),
                        );
                        //TODO: Resolve deps and create a lock file and return contents later.
                        todo!()
                    };

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
                    let dry_run = !sub_matches.is_present("dry_run");

                    // TODO: Implement this later..
                    if sub_matches.is_present("registry") {
                        let registry_path = sub_matches.get_one::<String>("registry");
                        // TODO: Implement this.
                    }

                    // TODO: Handle these errors properly.
                    let submission = reqwest::blocking::multipart::Form::new()
                        .text("public", if public { "true" } else { "false" })
                        .file("field", field_file_path)?
                        .file("lock_file", lock_file_path)?;

                    let client = reqwest::blocking::Client::new();

                    // Read auth.
                    let token = "";

                    let submission_response = client
                        .post("your url")
                        .header("Authorization", &format!("Basic {token}"))
                        .multipart(submission)
                        .send()?;
                    let status = submission_response.status();

                    if !status.is_success() {
                        if let Ok(response_body_value) =
                            submission_response.json::<serde_json::Value>()
                        {
                            if let Some(status_text_value) = response_body_value.get("status") {
                                let response_body_contents = response_body_value.to_string();
                                if let Some(status_text) = status_text_value.as_str() {
                                    if let Ok(status_info) = StatusInfo::try_from(status_text) {
                                        let negative_response = match status_info {
                                            StatusInfo::Error => {
                                                if let Ok(error_response) = serde_json::from_str::<
                                                    crate::submit::response::Error,
                                                >(
                                                    &response_body_contents,
                                                ) {
                                                    crate::submit::response::PlowSubmissionErrorResponse::Error(error_response)
                                                } else {
                                                    // Invalid response
                                                    todo!()
                                                }
                                            }
                                            StatusInfo::Failure => {
                                                if let Ok(failure_response) = serde_json::from_str::<
                                                    crate::submit::response::Failure,
                                                >(
                                                    &response_body_contents,
                                                ) {
                                                    crate::submit::response::PlowSubmissionErrorResponse::Failure(failure_response)
                                                } else {
                                                    // Invalid response
                                                    todo!()
                                                }
                                            }
                                            _ => {
                                                // Can not be success here..
                                                todo!()
                                            }
                                        };

                                        // TODO: Maybe the enum in between is not necessary..
                                        // Do something with the failure..
                                        todo!()
                                    } else {
                                        todo!()
                                        // not jsend
                                    }
                                } else {
                                    todo!()
                                    // Bad text
                                }
                            } else {
                                todo!()
                                // Bad response.
                            }
                        } else {
                            command_failed(&format!("Submission failed with status code {status}. There was no valid body in the response."));
                        }
                    }

                    println!(
                        "\t{} successful. The field is now uploaded to plow.",
                        "Submission".green().bold(),
                    );
                    return Ok(());
                }
                command_failed(
                            &format!("The field you've provided at {field_file_path} is not readable, you may check that if the file is readable in a normal text editor first."),
                        );
            }
            bail!("Please give a file path to a ttl file to lint.");
        }

        _ => {
            let path_to_plow_toml = camino::Utf8PathBuf::from("./Plow.toml");
            let path_to_fields_dir = camino::Utf8PathBuf::from("./fields");
            let existing_field_paths_in_directory = util::list_files(".", "ttl");
            if existing_field_paths_in_directory.is_empty() && !path_to_fields_dir.exists() {
                command_failed(
                    "please run this command in a directory containing .ttl files in any depth",
                );
            }
            let found_field_metadata: Vec<FieldMetadata> = existing_field_paths_in_directory
                .iter()
                .map(|p| get_a_list_of_requested_dependencies_from_a_field(&p.to_string_lossy()))
                .collect();

            // Create fields directory if it does not exist.
            std::fs::create_dir(&path_to_fields_dir);

            for (existing_path, field_metadata) in existing_field_paths_in_directory
                .iter()
                .zip(found_field_metadata.iter())
            {
                std::fs::create_dir(path_to_fields_dir.join(&field_metadata.namespace));
                std::fs::create_dir(
                    path_to_fields_dir
                        .join(&field_metadata.namespace)
                        .join(&field_metadata.name),
                );
                let new_field_dest = path_to_fields_dir
                    .join(&field_metadata.namespace)
                    .join(&field_metadata.name)
                    .join(
                        &existing_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    );
                std::fs::copy(existing_path, &new_field_dest)?;
            }

            let field_paths_in_fields_dir = util::list_files(path_to_fields_dir, "ttl");

            let found_field_metadata: Vec<FieldMetadata> = field_paths_in_fields_dir
                .iter()
                .map(|p| get_a_list_of_requested_dependencies_from_a_field(&p.to_string_lossy()))
                .collect();

            let workspace: config::Workspace = field_paths_in_fields_dir.into();
            let config_file = toml::to_string::<PlowConfigFile>(
                &config::PlowConfigFile::with_workspace(&workspace),
            )
            .unwrap();

            std::fs::write(path_to_plow_toml, config_file);

            let organizations_to_resolve_for = found_field_metadata
                .iter()
                .cloned()
                .map(|meta| meta.into())
                .collect::<Vec<OrganizationToResolveFor>>();

            // Resolve deps and lock, prepare protege ws
        }
    }

    Ok(())
}

fn command_failed(advice: &str) {
    println!("\t{}", "Command failed".red().bold(),);
    println!("\t{} {advice}", "Advice".yellow().bold(),);
    std::process::exit(0xFF);
}

fn command_not_complete(advice: &str) {
    println!("\t{}", "Command is not complete".red().bold(),);
    println!("\t{} {advice}", "Advice".yellow().bold(),);
    std::process::exit(0xFF);
}

// TODO: Some more explanation about what kind of input this function expects.
fn initialize() -> Result<()> {
    let ontology_name = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(
            "Name of the ontology? (internal name, only alphanumeric characters and underscores)",
        )
        .validate_with(|input: &String| validate_ontology_name(input))
        .interact_text()?;
    let ontology = initialize_ontology(&ontology_name)?;
    print!("{ontology}");
    Ok(())
}
fn lint_file(ontology_file_path: &str, specific_lints: Option<Vec<Box<dyn Lint>>>) -> Result<()> {
    let ontology = std::fs::read_to_string(&ontology_file_path).expect("Unable to read the file");

    let document = TurtleDocument::parse::<VerboseError<&str>>(&ontology)
        .expect("Unable to parse the ontology")
        .1;

    let lints = if let Some(specific_lints) = specific_lints {
        specific_lints
    } else {
        all_lints()
    };

    let mut contains_err = false;
    for lint in lints {
        use LintResult::*;
        let res = lint.lint(&document);
        match res {
            Success(message) => {
                println!("{}", message.green());
            }
            Warning(messages) => {
                for message in messages {
                    println!("{}", message.yellow());
                }
            }
            Failure(messages) => {
                for message in messages {
                    println!("{}", message.red());
                }
                contains_err = true;
            }
        }
    }

    if contains_err {
        return Err(anyhow!(
            "Lints were not all successful. The field contains errors."
        ));
    }
    Ok(())
}
