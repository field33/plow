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

pub mod constants;
use anyhow::{anyhow, bail};

#[derive(Debug, Default)]
pub struct PackageName(String);

impl ToString for PackageName {
    fn to_string(&self) -> String {
        format!(
            "{namespace}/{name}",
            namespace = self.namespace(),
            name = self.package()
        )
    }
}

#[allow(clippy::indexing_slicing)]
impl PackageName {
    /// Returns the namespace of the package name (including `@` prefix).
    pub fn namespace(&self) -> &str {
        let parts = self.0.split('/').collect::<Vec<_>>();
        parts[0]
    }

    /// Returns the name of the package.
    pub fn package(&self) -> &str {
        let parts = self.0.split('/').collect::<Vec<_>>();
        parts[1]
    }
}

#[allow(clippy::indexing_slicing)]
impl TryFrom<String> for PackageName {
    type Error = anyhow::Error;
    fn try_from(package_name: String) -> Result<Self, Self::Error> {
        // TODO: verify allowed characters
        let parts = package_name.split('/').collect::<Vec<_>>();
        if parts.len() != 2 {
            bail!("Invalid package name `{package_name}` - Package name must consist of namespace and per-namespace-name", package_name = package_name );
        }
        if parts[0]
            .chars()
            .next()
            .ok_or_else(|| anyhow!("Package name prefix doesn't consist of any characters"))?
            != '@'
        {
            bail!("Invalid namespace `{namespace}` of `{package_name}` - Namespace must have `@` as a first character", package_name = package_name, namespace= parts[0] );
        }
        Ok(Self(package_name))
    }
}
