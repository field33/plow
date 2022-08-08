use anyhow::{anyhow, Result};
use clap::ArgMatches;
use clap::{arg, App, Command};
use colored::*;
use dialoguer::console::Emoji;
use harriet::TurtleDocument;
use nom::error::VerboseError;
use plow_linter::{
    lint::{Lint, LintResult},
    lints::all_lints,
};

use crate::feedback::command_not_complete;

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("lint")
        .about("Lints a field.")
        .arg(arg!([FIELD_PATH]))
}

pub fn run_command(sub_matches: &ArgMatches) -> Result<()> {
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
        // Unreachable code.
        return Ok(());
    }
    command_not_complete("please provide a field (a valid .ttl file path) for plow to lint");
    // Unreachable code.
    return Ok(());
}

pub fn lint_file(
    ontology_file_path: &str,
    specific_lints: Option<Vec<Box<dyn Lint>>>,
) -> Result<()> {
    let ontology = std::fs::read_to_string(&ontology_file_path).expect("Unable to read the file");

    let document = TurtleDocument::parse::<VerboseError<&str>>(&ontology)
        .expect("Unable to parse the ontology")
        .1;

    let lints = specific_lints.map_or_else(all_lints, |specific_lints| specific_lints);

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
