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
mod feedback;
mod init;
mod lint;
mod login;
mod submit;
mod workspace;

use anyhow::Result;
use clap::App;

pub fn main() -> Result<()> {
    let matches = App::new("plow")
        .version("0.1.0")
        .author("Ali Somay <ali@field33.com>")
        .about("Plow package manager.")
        .subcommand(lint::attach_as_sub_command())
        .subcommand(login::attach_as_sub_command())
        .subcommand(submit::attach_as_sub_command())
        .subcommand(init::attach_as_sub_command())
        .get_matches();

    match matches.subcommand() {
        Some(("login", sub_matches)) => {
            login::run_command(sub_matches)?;
        }
        Some(("lint", sub_matches)) => {
            lint::run_command(sub_matches)?;
        }
        Some(("submit", sub_matches)) => {
            submit::run_command(sub_matches)?;
        }
        Some(("init", sub_matches)) => {
            init::run_command(sub_matches)?;
        }
        _ => workspace::prepare()?,
    }

    Ok(())
}
