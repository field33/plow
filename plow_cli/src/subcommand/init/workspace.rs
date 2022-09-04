pub mod fields;

use self::fields::FieldsDirectory;
use crate::config::files::workspace_manifest::WorkspaceManifestFile;
use crate::config::PlowConfig;
use crate::{error::CliError, error::WorkspaceInitializationError::*};

pub fn prepare(config: &PlowConfig, force: bool) -> Result<(), CliError> {
    if config.working_dir.path.join("Plow.toml").exists() && !force {
        return Err(WorkspaceAlreadyInitialized.into());
    }

    if force {
        // Clean up before creation
        let manifest_file_path = config.working_dir.path.join("Plow.toml");

        if manifest_file_path.exists() {
            std::fs::remove_file(&manifest_file_path)
                .map_err(|err| FailedToRemoveWorkspaceManifestFile(err.to_string()))?;
        }

        // Clean up before creation
        let lock_file_path = config.working_dir.path.join("Plow.lock");

        if lock_file_path.exists() {
            std::fs::remove_file(&lock_file_path)
                .map_err(|err| FailedToRemoveWorkspaceManifestFile(err.to_string()))?;
        }
    }

    let fields_dir_path = config.working_dir.path.join("fields");
    if fields_dir_path.exists() {
        FieldsDirectory::explode(&fields_dir_path, config)?;
    }

    let mut fields_dir = FieldsDirectory::empty_with_path(&fields_dir_path);

    fields_dir.children =
        crate::utils::list_files(".", "ttl").map_err(|err| FailedRecursiveListingFields {
            reason: err.to_string(),
        })?;

    if fields_dir.children.is_empty() && !fields_dir.exists_in_filesystem() {
        return Err(NoFieldsInDirectory.into());
    }

    let linting_failures = fields_dir.lint_all_children();

    // Remove the paths from the list of found fields which has failed lints.
    if let Some((ref failed_paths, _)) = linting_failures {
        fields_dir
            .children
            .retain(|path| !failed_paths.contains(&path.to_string()));
    }

    // Create fields directory and fill with children if not exists already.
    if !fields_dir.exists_in_filesystem() {
        fields_dir.write_with_children()?;
    }

    // Now that we filtered and collected all the fields lets create the workspace manifest file.
    let workspace_manifest_file = WorkspaceManifestFile::from(&fields_dir);
    workspace_manifest_file.write()?;

    // -------

    // TODO:  Here we'd need to create initial entrypoint index like structures (Organizations~)
    // To feed to the dependency resolution.
    // It is just an easy iteration.

    // ------- Dependency resolution ------- may start
    // But actually the real workspace construction starts here.
    // We're done with the first phase of gathering information and writing files.

    // let _organizations_to_resolve_for = field_metadata_in_fields_dir
    //     .iter()
    //     .cloned()
    //     .map(std::convert::Into::into)
    //     .collect::<Vec<OrganizationToResolveFor>>();

    if let Some((_, err)) = linting_failures {
        return Err(err);
    }

    // Prepare workspace (organization folder creation, Plow toml etc. acquire the list of dependencies to resolve to create lock files)
    // Done mostly
    // Clone or update the public index (currently) and provide it as a registry to lock the workspace.
    // Progress...
    // Start dependency resolution and write lock files.
    // Do the protege part if a command line arg is provided.
    // Always update the index with some plow commands.
    Ok(())
}
