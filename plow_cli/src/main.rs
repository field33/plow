#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,
    clippy::non_ascii_literal,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit,
    // We decided that we're ok with expect
    clippy::expect_used,
    clippy::wildcard_imports,
    clippy::as_conversions,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,
    // Currently breaks CI, let's wait a bit more until new clippy version is more spread.
    // clippy::single_char_lifetime_names,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    clippy::wildcard_dependencies,
    clippy::redundant_feature_names,
    clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,

    clippy::negative_feature_names
)]

pub mod config;
mod error;
mod feedback;
pub mod git;
pub mod manifest;

pub mod resolve;
mod subcommand;
pub mod sync;
pub mod utils;

use camino::Utf8PathBuf;
use clap::{App, AppSettings, Arg};
use feedback::{command_failed, Feedback};

#[allow(clippy::missing_panics_doc)]
pub fn main() {
    let app = App::new("plow")
        .version("0.4.6")
        .about("Plowing the field of knowledge. Package management for ontologies.")
        .arg(
            Arg::with_name("registry")
                .value_name("url")
                .long("registry")
                .help("Specifies the target registry for subcommands which interact with it.")
                .takes_value(true),
        )
        // .arg(
        //     Arg::with_name("fetch-with-cli")
        //         .long("fetch-with-cli")
        //         .help("Uses the host git application to fetch private index.")
        //         .takes_value(false)
        //         .action(clap::ArgAction::SetTrue),
        // )
        .arg(
            Arg::with_name("config")
                .value_name("directory")
                .long("config")
                .help("Specify a different home for plow. The path specified here will override the default config directory (~/.plow).")
                .takes_value(true),
        )
        .subcommand(subcommand::lint::attach_as_sub_command())
        .subcommand(subcommand::login::attach_as_sub_command())
        .subcommand(subcommand::submit::attach_as_sub_command())
        .subcommand(subcommand::init::attach_as_sub_command())
        .subcommand(subcommand::update::attach_as_sub_command())
        .subcommand(subcommand::protege::attach_as_sub_command())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::SubcommandPrecedenceOverArg);

    let options = app.clone().get_matches();

    let custom_plow_home_path = options.get_one::<String>("config").map(Utf8PathBuf::from);
    let custom_registry_url = options.get_one::<String>("registry").cloned();
    // let custom_registry_url = None;
    // let fetch_with_cli = options.get_flag("fetch-with-cli");
    // TODO: This option is not read anymore. And will be removed soon.
    let fetch_with_cli = false;

    let matches = app.clone().get_matches();
    match config::configure(custom_plow_home_path, custom_registry_url, fetch_with_cli) {
        Ok(ref config) => {
            let mut app_for_help_reference = app.clone();

            if match matches.subcommand() {
                Some(("login", sub_matches)) => {
                    subcommand::login::run_command(sub_matches, config).feedback();
                    Some(())
                }
                Some(("lint", sub_matches)) => {
                    subcommand::lint::run_command(sub_matches, config).feedback();
                    Some(())
                }
                Some(("submit", sub_matches)) => {
                    subcommand::submit::run_command(sub_matches, config).feedback();
                    Some(())
                }
                Some(("init", sub_matches)) => {
                    subcommand::init::run_command(sub_matches, config).feedback();
                    Some(())
                }
                Some(("update", sub_matches)) => {
                    subcommand::update::run_command(sub_matches, config).feedback();
                    Some(())
                }
                Some(("protege", sub_matches)) => {
                    subcommand::protege::run_command(sub_matches, config).feedback();
                    Some(())
                }
                _ => None,
            }
            .is_none()
                && app_for_help_reference.print_long_help().is_err()
            {
                command_failed("Please use a subcommand which is supported by this version of plow. You may consult plow --help.");
            }
        }
        Err(cli_error) => {
            cli_error.feedback();
        }
    }
}
