use clap::ArgMatches;
use clap::{arg, App, Command};
use colored::*;
use harriet::TurtleDocument;
use nom::error::VerboseError;
use plow_linter::{
    lint::{Lint, LintResult},
    lints::all_lints,
};

use crate::error::CliError;
use crate::error::LintSubcommandError::*;
use crate::feedback::{general_lint_start, general_lint_success, Feedback};

pub struct SuccessfulLint;
impl Feedback for SuccessfulLint {
    fn feedback(&self) {
        general_lint_success();
    }
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("lint")
        .about("Lints a field.")
        .arg(arg!([FIELD_PATH]))
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches) -> Box<dyn Feedback + '_> {
    match run_command_flow(sub_matches) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub fn run_command_flow(sub_matches: &ArgMatches) -> Result<impl Feedback, CliError> {
    let field_file_path = sub_matches
        .get_one::<String>("FIELD_PATH")
        .ok_or(NoFieldProvidedToLint)?;

    let field = camino::Utf8PathBuf::from(field_file_path);

    if field.exists() {
        general_lint_start();
        // TODO: Add specific lints here.
        lint_file(field_file_path, None)?;
        return Ok(SuccessfulLint);
    }

    Err(FailedToFindFieldToLint {
        field_path: field.into(),
    }
    .into())
}

pub fn lint_file(
    ontology_file_path: &str,
    specific_lints: Option<Vec<Box<dyn Lint>>>,
) -> Result<(), CliError> {
    let field_contents =
        std::fs::read_to_string(&ontology_file_path).map_err(|err| FailedToReadField {
            field_path: ontology_file_path.to_owned(),
            details: err.to_string(),
        })?;

    let (_, document) = TurtleDocument::parse::<VerboseError<&str>>(&field_contents)
        .map_err(|_| FailedToParseField)?;

    let lints = specific_lints.map_or_else(all_lints, |specific_lints| specific_lints);

    let mut contains_failures = false;
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
                contains_failures = true;
            }
        }
    }

    if contains_failures {
        return Err(LintContainsFailures.into());
    }
    Ok(())
}
