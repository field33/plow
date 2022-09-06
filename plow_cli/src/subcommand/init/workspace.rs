pub mod fields;

use plow_package_management::lock::LockFile;
use plow_package_management::package::PackageVersionWithRegistryMetadata;
use plow_package_management::registry::Registry;

use self::fields::FieldsDirectory;
use crate::config::files::workspace_manifest::WorkspaceManifestFile;
use crate::config::PlowConfig;
use crate::resolve::resolve;
use crate::{error::CliError, error::WorkspaceInitializationError::*};

pub fn prepare(config: &PlowConfig, force: bool) -> Result<(), CliError> {
    if config.working_dir.path.join("Plow.toml").exists() && !force {
        return Err(WorkspaceAlreadyInitialized.into());
    }

    if force {
        // Clean up before creation if running in a workspace
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
    let maybe_backed_up_fields_dir_path =
        FieldsDirectory::backup_if_already_exists(&fields_dir_path, config)?;

    let mut fields_dir =
        if let Some(ref backed_up_fields_dir_path) = maybe_backed_up_fields_dir_path {
            let mut dir = FieldsDirectory::fill_from_backup(backed_up_fields_dir_path)?;
            // We also extend from the working dir, not only checking backups dir, maybe new fields are added.
            // TODO: Do we need to check workspace root also?
            dir.extend_from_root_excluding_fields_dir(&config.working_dir.path)?;
            dir
        } else {
            FieldsDirectory::fill_from_root(&config.working_dir.path)?
        };

    if fields_dir.children.is_empty() && !fields_dir.exists_in_filesystem() {
        return Err(NoFieldsInDirectory.into());
    }

    let linting_failures = fields_dir.lint_all_children();

    // Remove the paths from the list of found fields which has failed lints.
    if let Some((ref failed_paths, _)) = linting_failures {
        fields_dir
            .children
            .retain(|path| !failed_paths.contains(&path.as_path().to_string()));
    }

    // Remove if there are duplicate paths. Which is unlikely and probably this is unnecessary.
    fields_dir.dedup();

    if force && fields_dir.exists_in_filesystem() {
        // It is backed up in an earlier stage.
        // Safe to remove.
        fields_dir.remove()?;
    }

    // Create fields directory and fill with children if not exists already.
    if !fields_dir.exists_in_filesystem() {
        fields_dir.write_with_children()?;
    }

    // Now that we filtered and collected all the fields lets create the workspace manifest file.
    let workspace_manifest_file = WorkspaceManifestFile::from(&fields_dir);
    workspace_manifest_file.write()?;

    if let Some(ref backed_up_fields_dir_path) = maybe_backed_up_fields_dir_path {
        // Remove the backed up fields directory.
        std::fs::remove_dir_all(backed_up_fields_dir_path)
            .map_err(|err| FailedToRemoveBackupFieldsDirectory(err.to_string()))?;
    }

    let registry = crate::sync::sync(config)?;

    let lock_file = fields_dir
        .children
        .iter()
        .fold(LockFile::default(), |mut lock_file, child| {
            if let Ok(Some(fresh_lock_file)) =
                resolve(config, child.as_path(), &registry as &dyn Registry)
            {
                lock_file
                    .locked_dependencies
                    .packages
                    .extend(fresh_lock_file.locked_dependencies.packages);

                return lock_file;
            }

            // Add the current package to the registry?

            lock_file
        });

    let resolved_dependencies_with_metadata: Vec<PackageVersionWithRegistryMetadata> = lock_file
        .locked_dependencies
        .packages
        .iter()
        .map(|package_version| registry.get_package_version_metadata(package_version))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| CliError::Wip(err.to_string()))?;

    if !resolved_dependencies_with_metadata.is_empty() {
        LockFile::write(
            Some(config.working_dir.path.clone()),
            &resolved_dependencies_with_metadata,
        )
        .map_err(|err| CliError::Wip(err.to_string()))?;
    }

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
