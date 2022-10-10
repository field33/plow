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
use crate::resolve::resolve;
use plow::manifest::FieldManifest;

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
