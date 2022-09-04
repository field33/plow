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
mod subcommand;
pub mod sync;
pub mod utils;

use clap::{App, AppSettings};
use feedback::{command_failed, Feedback};

#[allow(clippy::missing_panics_doc)]
pub fn main() {
    let app = App::new("plow")
        .version("0.2.2")
        .about("Plowing the field of knowledge. Package management for ontologies.")
        .subcommand(subcommand::lint::attach_as_sub_command())
        .subcommand(subcommand::login::attach_as_sub_command())
        .subcommand(subcommand::submit::attach_as_sub_command())
        .subcommand(subcommand::init::attach_as_sub_command())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::SubcommandPrecedenceOverArg);

    let custom_plow_home_path: Option<camino::Utf8PathBuf> = None;
    // TODO: Check for --config flag to retrieve a custom config dir path later.
    // So we can pass also Some values.

    let mut registry_url_override_from_command: Option<String> = None;
    let matches = app.clone().get_matches();
    if let Some((_, matches)) = matches.subcommand() {
        registry_url_override_from_command = matches.get_one::<String>("REGISTRY_URL").cloned();
    }

    match config::configure(custom_plow_home_path, registry_url_override_from_command) {
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
