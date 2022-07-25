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

    // We decided that we're ok with expect
    clippy::expect_used,

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
mod config;
mod login;
mod submit;

use anyhow::{anyhow, bail, Result};
use clap::{arg, App, Arg, Command};
use config::create_configuration_directory_if_not_exists;
use dialoguer::console::Emoji;
use dialoguer::Input;
use harriet::TurtleDocument;
use nom::error::VerboseError;
use plow_linter::lint::LintResult;
use plow_ontology::{initialize_ontology, validate_ontology_name};
// This is currently fine for this stage of this binary.
#[allow(clippy::wildcard_imports)]
use plow_linter::lints::*;

use crate::login::{get_saved_api_token, save_credentials_replace_existing};
use colored::*;

pub static SUCCESS: Emoji = Emoji("✅  ", "SUCCESS");
pub static WARNING: Emoji = Emoji("⚠️  ", "MAYBE");
pub static FAILURE: Emoji = Emoji("❌  ", "FAILURE");

// TODO: Make a scope and a development plan for this project, it is currently just a sketch.
pub fn main() -> Result<()> {
    let matches = App::new("Ontology Tools CLI")
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
                    "Login".green(),
                );
                return Ok(());
            }
            println!("\t{}", "Command is not complete".red(),);
            println!(
                "\t{} please provide a valid API token to save",
                "Advice".yellow(),
            );
            return Ok(());
        }
        Some(("lint", sub_matches)) => {
            if let Some(file_path) = sub_matches.get_one::<String>("FIELD_PATH") {
                lint_file(file_path)?;
            } else {
                bail!("Please give a file path to a ttl file to lint.");
            }
        }
        Some(("submit", sub_matches)) => {
            // TODO: Implement this.
            if let Some(file_path) = sub_matches.get_one::<String>("FIELD_PATH") {
                // lint_file(file_path)?;
                if sub_matches.is_present("registry") {
                    let registry_path = sub_matches.get_one::<String>("registry");
                    bail!("All good.");
                }
                bail!("Please give a registry path to submit to.");
            }
            bail!("Please give a file path to a ttl file to lint.");
        }

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }

    Ok(())
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
fn lint_file(ontology_file_path: &str) -> Result<()> {
    let ontology = std::fs::read_to_string(&ontology_file_path).expect("Unable to read the file");

    let document = TurtleDocument::parse::<VerboseError<&str>>(&ontology)
        .expect("Unable to parse the ontology")
        .1;

    let lints = all_lints();

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
        return Err(anyhow!("The file contains errors."));
    }
    Ok(())
}
