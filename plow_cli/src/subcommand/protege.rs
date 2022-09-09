use crate::{
    config::PlowConfig,
    error::CliError,
    feedback::{field_info, Feedback},
    manifest::FieldManifest,
    resolve::resolve,
};

use camino::Utf8PathBuf;
use clap::{arg, App, AppSettings, ArgMatches, Command};
use colored::*;
use plow_linter::lints::all_lints;
use plow_package_management::registry::Registry;

use crate::error::FieldAccessError::*;

use super::lint::lint_file;

pub struct SuccessfulProtege;
impl Feedback for SuccessfulProtege {
    fn feedback(&self) {
        println!(
            "\t{} successfully opened in protege.",
            "Field".green().bold(),
        );
    }
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("protege")
        .about("Opens a field in protege.")
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
    sub_matches: &clap::ArgMatches,
    config: &PlowConfig,
) -> Result<impl Feedback, CliError> {
    let workspace_root = config.working_dir.fail_if_not_under_a_workspace()?;
    let field_file_path = sub_matches.get_one::<String>("FIELD_PATH").unwrap();
    // .ok_or(NoFieldProvidedToLint)?;

    let field_file_path = camino::Utf8PathBuf::from(field_file_path);
    field_info(&field_file_path)?;

    if field_file_path.exists() {
        lint_file(&field_file_path.to_string(), all_lints())?;

        let path = Utf8PathBuf::from(field_file_path);

        let registry = crate::sync::sync(config).map_err(|err| CliError::Wip(err.to_string()))?;

        let root_field_contents = std::fs::read_to_string(&path).map_err(|_| {
            CliError::from(FailedToFindFieldAtPath {
                field_path: path.to_string(),
            })
        })?;
        let root_field_manifest =
            FieldManifest::new(root_field_contents.clone()).map_err(|_| {
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

            // TODO: Protege related stuff..
        }

        return Ok(SuccessfulProtege);
    }

    Err(FailedToFindFieldAtPath {
        field_path: field_file_path.into(),
    }
    .into())
}
