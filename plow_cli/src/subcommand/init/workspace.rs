mod manifest;

use crate::{
    config::PlowConfigFile,
    error::CliError,
    error::WorkspaceInitializationError::*,
    subcommand::{
        init::{utils::list_files, workspace::manifest::FieldManifest},
        lint::lint_file_fail_on_failure,
    },
};
use camino::{Utf8Path, Utf8PathBuf};
use plow_package_management::package::{FieldMetadata, OrganizationToResolveFor};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;

// Prepare workspace (organization folder creation, Plow toml etc. acquire the list of dependencies to resolve to create lock files)
// Clone or update the public index (currently) and provide it as a registry to lock the workspace.
// Start dependency resolution and write lock files.
// Do the protege part if a command line arg is provided.
// Always update the index with some plow commands.

fn lint_found_fields(
    found_field_paths_in_directory: &[Utf8PathBuf],
) -> Option<(Vec<String>, CliError)> {
    // Lint all fields in the directory and collect failures if there are some.
    let failed_field_paths_on_linting = found_field_paths_in_directory
        .par_iter()
        .filter_map(|path| {
            if let Err(err) = lint_file_fail_on_failure(path.as_ref(), None) {
                Some(err)
            } else {
                None
            }
        })
        .filter_map(|err| match err {
            CliError::LintSubcommand(
                crate::error::LintSubcommandError::SingleLintContainsFailure { field_path },
            ) => Some(field_path),
            _ => None,
        })
        .collect::<Vec<_>>();

    // If there are failures, prepare the error to inform the user later.
    if !failed_field_paths_on_linting.is_empty() {
        return Some((
            failed_field_paths_on_linting.clone(),
            crate::error::LintSubcommandError::LintsContainFailures {
                field_paths: failed_field_paths_on_linting,
            }
            .into(),
        ));
    }
    None
}

fn create_fields_directory_from_found_fields(
    fields_dir: &Utf8Path,
    found_fields: &[FoundFieldInDirectory],
) -> Result<(), CliError> {
    std::fs::create_dir_all(fields_dir)
        .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

    for FoundFieldInDirectory {
        path: current_field_path,
        metadata: FieldMetadata {
            namespace, name, ..
        },
    } in found_fields
    {
        std::fs::create_dir_all(fields_dir.join(&namespace))
            .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

        std::fs::create_dir_all(fields_dir.join(&namespace).join(&name))
            .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

        #[allow(clippy::unwrap_used)]
        let new_field_destination = fields_dir
            .join(&namespace)
            .join(&name)
            // Checked before
            .join(&current_field_path.file_name().unwrap());

        std::fs::copy(current_field_path, &new_field_destination)
            .map_err(|err| FailedToWriteToFieldsDirectory(err.to_string()))?;
    }
    Ok(())
}

#[derive(Debug)]
struct FoundFieldInDirectory {
    pub path: Utf8PathBuf,
    pub metadata: FieldMetadata,
}

pub fn prepare() -> Result<(), CliError> {
    let plow_toml = Utf8PathBuf::from("./Plow.toml");
    let fields_dir = Utf8PathBuf::from("./fields");

    let mut found_field_paths_in_directory =
        list_files(".", "ttl").map_err(|err| FailedRecursiveListingFields {
            reason: err.to_string(),
        })?;

    if found_field_paths_in_directory.is_empty() && !fields_dir.exists() {
        return Err(NoFieldsInDirectory.into());
    }

    let linting_failures = lint_found_fields(&found_field_paths_in_directory);

    // Remove the paths from the list of found fields which has failed lints.
    if let Some((failed_paths, _)) = linting_failures {
        found_field_paths_in_directory.retain(|path| !failed_paths.contains(&path.to_string()));
    }

    #[allow(clippy::unwrap_used)]
    let found_fields_in_directory = found_field_paths_in_directory
        .iter()
        // Assume linted
        .map(|path| (path, FieldManifest::new(path.to_string()).unwrap()))
        .map(|(path, manifest)| {
            let metadata = manifest.make_field_metadata_from_manifest_unchecked();
            FoundFieldInDirectory {
                path: path.clone(),
                metadata,
            }
        })
        .collect::<Vec<_>>();

    // Create fields directory if it does not exist.
    if !fields_dir.exists() {
        create_fields_directory_from_found_fields(&fields_dir, &found_fields_in_directory)?;
    }

    let field_paths_in_fields_dir =
        list_files(&fields_dir, "ttl").map_err(|err| FailedRecursiveListingFields {
            reason: err.to_string(),
        })?;

    #[allow(clippy::unwrap_used)]
    let field_metadata_in_fields_dir = found_field_paths_in_directory
        .iter()
        // Assume linted
        .map(|path| FieldManifest::new(path.to_string()).unwrap())
        .map(|manifest| manifest.make_field_metadata_from_manifest_unchecked())
        .collect::<Vec<_>>();

    let workspace: crate::config::Workspace = field_paths_in_fields_dir.into();

    // We specify the file this can not fail.
    #[allow(clippy::unwrap_used)]
    let config_file =
        toml::to_string::<PlowConfigFile>(&PlowConfigFile::with_workspace(&workspace)).unwrap();

    std::fs::write(&plow_toml, config_file)
        .map_err(|err| FailedToCreatePlowToml(err.to_string()))?;

    let _organizations_to_resolve_for = field_metadata_in_fields_dir
        .iter()
        .cloned()
        .map(std::convert::Into::into)
        .collect::<Vec<OrganizationToResolveFor>>();

    // Prepare workspace (organization folder creation, Plow toml etc. acquire the list of dependencies to resolve to create lock files)
    // Clone or update the public index (currently) and provide it as a registry to lock the workspace.
    // Start dependency resolution and write lock files.
    // Do the protege part if a command line arg is provided.
    // Always update the index with some plow commands.
    Ok(())
}
