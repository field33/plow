pub mod field;
pub mod workspace;

use crate::config::PlowConfig;
use crate::error::CliError;
use crate::error::FieldInitializationError::*;
use crate::feedback::Feedback;
use anyhow::Result;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use clap::{App, Command};
use clap::{Arg, ArgMatches};
use colored::Colorize;

pub struct SuccessfulFieldInitialization;
impl Feedback for SuccessfulFieldInitialization {
    fn feedback(&self) {
        println!(
            "\t{} successful. Currently initialization prints the field to stdout. When workspace creation is done, the field will be saved to the workspace.",
            "Field creation".green().bold(),
        );
    }
}

pub struct SuccessfulWorkspaceCreation;
impl Feedback for SuccessfulWorkspaceCreation {
    fn feedback(&self) {
        println!("Work in progress..");
    }
}

fn initialize_field(
    field_name: &str,
    workspace_root: &Utf8Path,
) -> Result<impl Feedback, CliError> {
    // Overwrites existing

    let field = self::field::new(field_name);

    let file_name = format!("{}.ttl", field_name.split('/').last().unwrap());

    let p = workspace_root.join("fields").join(field_name);

    std::fs::create_dir_all(&p).map_err(|err| FailedToWriteField(err.to_string()))?;
    std::fs::write(p.join(file_name), field.as_bytes())
        .map_err(|err| FailedToWriteField(err.to_string()))?;

    field.split('\n').for_each(|line| println!("\t\t{}", line));
    Ok(SuccessfulFieldInitialization)
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("init")
        .about("Initializes and prepares a workspace.")
        .arg(
            Arg::with_name("field")
                .value_name("name")
                .long("field")
                .help("Initializes a field.")
                .takes_value(true),
        )
        .arg_required_else_help(false)
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches, config: &PlowConfig) -> Box<dyn Feedback + 'static> {
    match run_command_flow(sub_matches, config) {
        Ok(feedback) => feedback,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub struct SuccessfulWorkspaceInitialization;
impl Feedback for SuccessfulWorkspaceInitialization {
    fn feedback(&self) {
        println!("\t{} created successfully. ", "Workspace".green().bold(),);
    }
}

pub fn run_command_flow(
    sub_matches: &ArgMatches,
    config: &PlowConfig,
) -> Result<Box<dyn Feedback>, CliError> {
    if sub_matches.is_present("field") {
        let field_name = sub_matches
            .get_one::<String>("field")
            .ok_or(NoFieldNameProvided)?;

        let workspace_root = config.working_dir.fail_if_not_under_a_workspace()?;

        let success = initialize_field(field_name, &workspace_root)?;

        return Ok(Box::new(success) as Box<dyn Feedback>);
    }

    workspace::prepare(config)?;
    Ok(Box::new(SuccessfulWorkspaceInitialization) as Box<dyn Feedback>)
}
