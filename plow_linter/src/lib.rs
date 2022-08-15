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

use harriet::TurtleDocument;
use lint::{LintResult, LintResults};
use lints::{LintSet, PlowLint};
use plow_graphify::document_to_graph;

use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use rdftk_core::model::graph::GraphRef;

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

// We only read immutable data
// We don't cast or transmute anything
// We do not play with reference counting.
// This wrapper type is only Sync to allow parallel execution of the linting process.
// Underlying type is not sync because it's methods return `Rc<RefCell<..>>`
// On the other hand we only read from that type by doing one borrow at a time so it is not much different than &.
// TODO: Document this in a proper way.
#[allow(clippy::undocumented_unsafe_blocks)]
unsafe impl Sync for MultiReaderRdfGraph {}
pub struct MultiReaderRdfGraph {
    pub inner: GraphRef,
}

pub struct Linter<'linter> {
    document: TurtleDocument<'linter>,
    graph: MultiReaderRdfGraph,
    pub lints: Vec<PlowLint>,
    pub sub_lints: Option<Vec<PlowLint>>,
}

impl<'linter> TryFrom<&'linter str> for Linter<'linter> {
    type Error = anyhow::Error;
    fn try_from(field_contents: &'linter str) -> Result<Self, Self::Error> {
        let document = TurtleDocument::parse_full(field_contents)
            .map_err(|_| anyhow::anyhow!("Parse error."))?;
        let graph = document_to_graph(&document)?;
        let multi_reader_graph = MultiReaderRdfGraph { inner: graph };
        Ok(Self {
            document,
            graph: multi_reader_graph,
            lints: vec![],
            sub_lints: None,
        })
    }
}

// TODO: Maybe implement builder pattern later?
impl Linter<'_> {
    pub fn add_lint(&mut self, lint: PlowLint) {
        self.lints.push(lint);
    }
    pub fn add_lint_set(&mut self, lint: LintSet) {
        self.lints.extend(lint.lints);
        if let Some(provided_sub_lints) = lint.sub_lints {
            if let Some(ref mut existing_sub_lints) = self.sub_lints {
                existing_sub_lints.extend(provided_sub_lints);
            } else {
                self.sub_lints = Some(provided_sub_lints);
            }
        }
    }

    pub fn add_sub_lint(&mut self, sub_lint: PlowLint) {
        if let Some(ref mut existing_sub_lints) = self.sub_lints {
            existing_sub_lints.push(sub_lint);
        } else {
            self.sub_lints = Some(vec![sub_lint]);
        }
    }

    pub fn remove_lint_set(&mut self) {
        self.lints.clear();
        self.sub_lints = None;
    }
}

impl Linter<'_> {
    pub fn run_lints(&self) -> Vec<LintResult> {
        let results = self.lints.iter().fold(vec![], |mut results, lint| {
            let result = lint.run(self);
            results.push(result);
            results
        });
        results
    }
    pub fn run_lints_check_if_contains_any_failure(&self) -> bool {
        self.lints.iter().any(|lint| lint.run(self).is_failure())
    }
    pub fn run_lints_in_parallel(&self) -> Vec<LintResult> {
        let iterator = self.lints.par_iter();
        iterator
            .fold(std::vec::Vec::new, |mut results: Vec<LintResult>, lint| {
                if lint.can_run_in_parallel() {
                    let result = lint.run(self);
                    results.push(result);
                }
                results
            })
            .collect::<LintResults>()
            .results
    }
    pub fn run_lints_in_parallel_check_if_contains_any_failure(&self) -> bool {
        let iterator = self.lints.par_iter();
        iterator.any(|lint| {
            if lint.can_run_in_parallel() {
                lint.run(self).is_failure()
            } else {
                false
            }
        })
    }
    // TODO: These functions are a hack and intended to be removed later.
    pub fn run_lints_which_are_only_allowed_to_be_run_in_sequence(&self) -> Vec<LintResult> {
        let sequential_run_results = self.lints.iter().fold(vec![], |mut results, lint| {
            if !lint.can_run_in_parallel() {
                results.push(lint.run(self));
            }
            results
        });
        sequential_run_results
    }
    pub fn run_lints_which_are_only_allowed_to_be_run_in_sequence_check_if_contains_any_failure(
        &self,
    ) -> bool {
        let there_are_failures_in_sequential_run = self.lints.iter().any(|lint| {
            if lint.can_run_in_parallel() {
                false
            } else {
                lint.run(self).is_failure()
            }
        });
        there_are_failures_in_sequential_run
    }
    //
}
