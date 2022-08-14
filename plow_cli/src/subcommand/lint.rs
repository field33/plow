use clap::{arg, App, Command};
use clap::{AppSettings, ArgMatches};
use colored::*;
use plow_linter::lint::LintResult;
use plow_linter::lints::*;
use plow_linter::Linter;

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
        .setting(AppSettings::ArgRequiredElseHelp)
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
        lint_file(field_file_path, all_lints())?;
        return Ok(SuccessfulLint);
    }

    Err(FailedToFindFieldToLint {
        field_path: field.into(),
    }
    .into())
}

pub fn lint_file(field_path: &str, lints: LintSet) -> Result<(), CliError> {
    let field_contents = std::fs::read_to_string(field_path).map_err(|err| FailedToReadField {
        field_path: field_path.to_owned(),
        details: err.to_string(),
    })?;

    // TODO: Handle this error.
    let mut linter = Linter::try_from(field_contents.as_ref()).unwrap();
    linter.add_lint_set(lints);
    let results = linter.run_lints();

    let mut contains_failures = false;
    for result in results {
        use LintResult::*;
        match result {
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
        return Err(LintsContainFailuresOpaque.into());
    }
    Ok(())
}

pub fn lint_file_fail_on_failure(field_path: &str, lints: LintSet) -> Result<(), CliError> {
    let field_contents = std::fs::read_to_string(field_path).map_err(|err| FailedToReadField {
        field_path: field_path.to_owned(),
        details: err.to_string(),
    })?;

    // TODO: Handle this error.
    let mut linter = Linter::try_from(field_contents.as_ref()).unwrap();
    linter.add_lint_set(lints);

    if linter.run_lints_check_if_contains_any_failure() {
        return Err(SingleLintContainsFailure {
            field_path: field_path.to_owned(),
        }
        .into());
    }

    Ok(())
}
