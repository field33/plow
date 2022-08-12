mod field;

use crate::error::CliError;
use crate::error::FieldInitializationError::*;
use crate::feedback::Feedback;
use anyhow::Result;
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

fn initialize_field(field_name: &str) -> Result<impl Feedback, CliError> {
    let field = self::field::new(field_name)?;
    print!("{field}");
    Ok(SuccessfulFieldInitialization)
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("init")
        .about("Initializes and prepares a workspace.")
        .arg(
            Arg::with_name("field")
                .short('f')
                .value_name("FIELD_NAME")
                .long("field")
                .help("Initializes a field.")
                .takes_value(true),
        )
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches) -> Box<dyn Feedback + '_> {
    match run_command_flow(sub_matches) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub fn run_command_flow(sub_matches: &ArgMatches) -> Result<impl Feedback, CliError> {
    if !sub_matches.args_present() {
        // crate::workspace::prepare()
        todo!()
    }
    if sub_matches.is_present("field") {
        let field_name = sub_matches
            .get_one::<String>("field")
            .ok_or(NoFieldNameProvided)?;

        let success = initialize_field(field_name)?;

        return Ok(success);
    }
    Err(CliError::UnknownOption)
}
