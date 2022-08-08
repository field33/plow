use crate::feedback::{command_failed, command_not_complete};
use anyhow::Result;
use clap::{arg, App, Command};
use clap::{Arg, ArgMatches};
use plow_ontology::{initialize_ontology, validate_ontology_name};

fn initialize(field_name: &str) -> Result<()> {
    validate_ontology_name(field_name).map_err(|err| anyhow::anyhow!("{}", err))?;
    let ontology = initialize_ontology(field_name)?;
    print!("{ontology}");
    Ok(())
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

pub fn run_command(sub_matches: &ArgMatches) -> Result<()> {
    if !sub_matches.args_present() {
        super::workspace::prepare()
    } else if sub_matches.is_present("field") {
        if let Some(field_name) = sub_matches.get_one::<String>("field") {
            initialize(field_name)?;
            return Ok(());
        }
        command_not_complete("Please provide a field name for plow to initialize");
        Ok(())
    } else {
        command_failed("Please provide a valid option to plow init");
        Ok(())
    }
}
