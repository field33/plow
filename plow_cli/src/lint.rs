use anyhow::{anyhow, Result};
use clap::ArgMatches;
use clap::{arg, App, Command};
use colored::*;
use harriet::TurtleDocument;
use nom::error::VerboseError;
use plow_linter::{
    lint::{Lint, LintResult},
    lints::all_lints,
};

use crate::feedback::{command_not_complete, linting_failed};

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("lint")
        .about("Lints a field.")
        .arg(arg!([FIELD_PATH]))
}

pub fn run_command(sub_matches: &ArgMatches) {
    if let Some(file_path) = sub_matches.get_one::<String>("FIELD_PATH") {
        let path_buf = camino::Utf8PathBuf::from(file_path);
        if path_buf.exists() {
            println!("\t{} the provided field..", "Linting".green().bold(),);

            // TODO: Add specific lints here.
            if lint_file(file_path, None).is_err() {
                linting_failed();
            }

            println!("\t{} successful.", "Linting".green().bold(),);
        }
        command_not_complete(
                    &format!("please provide a field (a valid .ttl file path) for plow to lint, {file_path} does not exist"),
                );
    }
    command_not_complete("please provide a field (a valid .ttl file path) for plow to lint");
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
                println!("\t\t{}", message.green());
            }
            Warning(messages) => {
                for message in messages {
                    println!("\t\t{}", message.yellow());
                }
            }
            Failure(messages) => {
                for message in messages {
                    println!("\t\t{}", message.red());
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
