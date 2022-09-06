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
use crate::error::LintSubcommandError::*;
use crate::feedback::{field_info, general_lint_success, lint_start, Feedback};
use crate::resolve::resolve;

pub struct SuccessfulUpdate;
impl Feedback for SuccessfulUpdate {
    fn feedback(&self) {
        // TODO: Implement update command
        todo!()
    }
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("update")
        .about("Updates the registry index, caches dependencies and updates the lock file.")
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
    // TODO: Implement update command
    Ok(SuccessfulUpdate)
}
