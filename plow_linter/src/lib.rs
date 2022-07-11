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
    clippy::pub_use,
    clippy::format_push_string,

    // We decided that we're ok with expect
    clippy::expect_used,

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

//! # Package management
//!
//! In our dependency management system, there are 4 main "artifacts"
//!
//! - The input ontology file
//! - The ontology metadata (extracted from the ontology file), which includes among others:
//!   - The name of the ontology
//!   - The version of the ontology
//!   - The dependencies of the ontology (name + version specification of each)
//! - The dependency lockfile (generated from the metadata + data from the registry), which contains
//!   a set of resolved dependencies
//! - The materialized workspace (generated from lockfile + downloads from registry):
//!   - Modified input ontology file that contains OWL imports to the specified dependencies
//!   - Downloaded dependencies
//!   - Catalog file, that helps Protege to find the downloaded dependencies
//!
//! ## Process
//!
//! For a full documentation of the package management process see [`doc_process`].

pub mod lint;
pub mod lints;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Documentation of the processes executed by the package manager
///
/// Below you can find a rough state chart of how we can derive a workspace from a ontology file and it's intermediate states.
///
/// ## States:
///   - `O`: The ontology file that serves as input
///   - `L`: The workspace with resolved and locked dependencies
///   - `R`: The workspace with retrieved (= downloaded) dependencies
///   - `wsProtege`: Special-purpose workspace, which can be edited with protege
///
/// ## State transitions:
///
///   - `resolve`: `O --> L`:
///     - Extracting dependencies from ontology file and resolving them to specific versions via registry index
///     - Writing the resolved dependencies to a lockfile, which may be used as a starting point for dependency resolution in the future
///   - `retrieve`: `L --> R` - Retrieving (= downloading) resolved dependencies from registry artifact store
///   - `constructWsProtege`: `R --> wsProtege`:
///     - Create separate directory for protege workspace
///     - Create hard link to original ontology file
///     - Copy retrieved dependencies to directory
///     - Create catalog file that allows Protege to find dependencies from directory
///
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
/// stateDiagram-v2
///     state "O: Ontology file" as oFile
///     state "L: Resolved & locked workspace" as lWorkspace
///     state "R: Workspace with retrieved dependencies" as rWorkspace
///     state "wsProtege: Workspace openable with Protege" as pWorkspace
///
///     oFile --> lWorkspace: resolve
///     lWorkspace --> rWorkspace: retrieve
///     rWorkspace --> pWorkspace: constructWsProtege
/// ```
pub const fn doc_process() {}
