use camino::Utf8PathBuf;
use clap::{arg, App, Command};
use clap::{AppSettings, ArgMatches};
use colored::*;
use plow_linter::lint::LintResult;
use plow_linter::lints::*;
use plow_linter::Linter;
use plow_package_management::registry::Registry;

use crate::config::PlowConfig;
use crate::error::CliError;
use crate::error::FieldAccessError::*;
use crate::error::LintSubcommandError::*;
use crate::feedback::{field_info, general_lint_success, lint_start, Feedback};
use crate::manifest::FieldManifest;
use crate::resolve::resolve;
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
pub fn run_command(sub_matches: &ArgMatches, config: &PlowConfig) -> Box<dyn Feedback + 'static> {
    match run_command_flow(sub_matches, config) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub fn run_command_flow(
    sub_matches: &ArgMatches,
    config: &PlowConfig,
) -> Result<impl Feedback, CliError> {
    let field_file_path = sub_matches
        .get_one::<String>("FIELD_PATH")
        .ok_or(NoFieldProvidedToLint)?;

    let field = camino::Utf8PathBuf::from(field_file_path);

    field_info(&field)?;

    if field.exists() {
        lint_file(field_file_path, all_lints())?;

        let path = Utf8PathBuf::from(field_file_path);

        let registry = crate::sync::sync(config)?;

        let root_field_contents = std::fs::read_to_string(&path).map_err(|_| {
            CliError::from(FailedToFindFieldAtPath {
                field_path: path.to_string(),
            })
        })?;
        let root_field_manifest = FieldManifest::new(&root_field_contents).map_err(|_| {
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

        return Ok(SuccessfulLint);
    }

    Err(FailedToFindFieldToLint {
        field_path: field.into(),
    }
    .into())
}

pub fn lint_file(field_path: &str, lints: Vec<LintSet>) -> Result<(), CliError> {
    let field_contents = std::fs::read_to_string(field_path).map_err(|err| FailedToReadField {
        field_path: field_path.to_owned(),
        details: err.to_string(),
    })?;

    let mut contains_failures = false;
    let mut linter = Linter::try_from(field_contents.as_ref()).map_err(|_| FailedToParseField {
        field_path: field_path.to_owned(),
    })?;

    for lint_set in lints {
        let set_id = lint_set.id;
        let set_name = linter.add_lint_set(lint_set);

        lint_start(&set_name);

        let results = linter.run_lint_set(set_id);

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

    let mut linter = Linter::try_from(field_contents.as_ref()).map_err(|_| FailedToParseField {
        field_path: field_path.to_owned(),
    })?;

    linter.add_lint_set(lints);

    if linter.run_lints_check_if_contains_any_failure() {
        return Err(SingleLintContainsFailure {
            field_path: field_path.to_owned(),
        }
        .into());
    }

    Ok(())
}
