pub mod fields;

use std::collections::HashMap;

use plow_package_management::lock::{LockFile, PackageInLockFile};
use plow_package_management::registry::Registry;

use self::fields::FieldsDirectory;
use crate::config::files::workspace_manifest::WorkspaceManifestFile;
use crate::config::PlowConfig;
use crate::manifest::FieldManifest;
use crate::resolve::resolve;
use crate::{error::CliError, error::FieldAccessError::*, error::WorkspaceInitializationError::*};

use dialoguer::{theme::ColorfulTheme, Confirm};

#[allow(clippy::too_many_lines)]
pub fn prepare(config: &PlowConfig) -> Result<(), CliError> {
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Plow will restructure this folder looking for .ttl files and grouping them to another folder backing up the existing ones, would you like to continue?")
        .default(true)
        .interact()
        .unwrap()
    {
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

        let fields_dir_path = config.working_dir.path.join("fields");
        let maybe_backed_up_fields_dir_path =
            FieldsDirectory::backup_if_already_exists(&fields_dir_path, config)?;

        let mut fields_dir =
            if let Some(ref backed_up_fields_dir_path) = maybe_backed_up_fields_dir_path {
                let mut dir = FieldsDirectory::fill_from_backup(backed_up_fields_dir_path)?;
                // We also extend from the working dir, not only checking backups dir, maybe new fields are added.
                // TODO: Do we need to check workspace root also?
                dir.extend_from_root_excluding_fields_dir_and_plow_backup(&config.working_dir.path)?;
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

        if fields_dir.exists_in_filesystem() {
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

        // root -> (resolved_root, deps of root[including transative])
        let mut collection: HashMap<String, (PackageInLockFile, LockFile)> = HashMap::new();

        // @attention We also inject the dependencies of the root field into the lock file.
        for child in &fields_dir.children {
            let root_field_contents = std::fs::read_to_string(&child.as_path()).map_err(|_| {
                CliError::from(FailedToFindFieldAtPath {
                    field_path: child.as_path().to_string(),
                })
            })?;
            let root_field_manifest =
                FieldManifest::new(root_field_contents.clone()).map_err(|_| {
                    CliError::from(FailedToReadFieldManifest {
                        field_path: child.as_path().to_string(),
                    })
                })?;

            #[allow(clippy::unwrap_used)]
            let root_field_name = root_field_manifest.field_namespace_and_name().unwrap();
            let root_dep_names = root_field_manifest
                .field_dependency_names()
                .unwrap_or_default();

            if let Ok(Some(fresh_lock_file)) = resolve(
                config,
                &root_field_contents,
                &root_field_manifest,
                false,
                &registry as &dyn Registry,
            ) {
                // Unwrap is fine here we've linted the field before.
                #[allow(clippy::unwrap_used)]
                let root_as_index = root_field_manifest.make_index_from_manifest().unwrap();
                // Check for duplicate names
                if collection.get(&root_field_name).is_some() {
                    return Err(CliError::from(DuplicateFieldInWorkspace(root_field_name)));
                }
                collection.insert(
                    root_field_name.clone(),
                    (
                        PackageInLockFile {
                            name: root_as_index.name,
                            version: root_as_index.version,
                            ontology_iri: root_as_index.ontology_iri,
                            source: None,
                            cksum: Some(root_as_index.cksum),
                            dependencies: root_dep_names,
                            root: true,
                        },
                        fresh_lock_file,
                    ),
                );
            }
        }

        let lock_file_contents = collection
            .into_iter()
            .flat_map(|(_, (root, locked_deps))| {
                let mut v = vec![];
                v.push(root);
                let deps = locked_deps
                    .locked_dependencies
                    .packages
                    .iter()
                    .map(|package_version| {
                        // Safe here, we passed dep resolution.
                        #[allow(clippy::unwrap_used)]
                        let metadata = registry
                            .get_package_version_metadata(package_version)
                            .unwrap();
                        PackageInLockFile {
                            name: package_version.package_name.clone(),
                            version: package_version.version.clone(),
                            ontology_iri: metadata.ontology_iri.clone(),
                            source: None,
                            cksum: metadata.cksum.clone(),
                            dependencies: metadata
                                .dependencies
                                .iter()
                                .cloned()
                                .map(|dep| dep.full_name)
                                .collect(),
                            root: false,
                        }
                    })
                    .collect::<Vec<_>>();
                v.extend(deps);
                v
            })
            .collect::<Vec<_>>();

        if !lock_file_contents.is_empty() {
            LockFile::write(Some(config.working_dir.path.clone()), &lock_file_contents)
                .map_err(|err| CliError::Wip(err.to_string()))?;
        }

        if let Some((_, err)) = linting_failures {
            return Err(err);
        }

        // TODO: Do the protege part if a command line arg is provided.
        // TODO: DO THE PROTEGE PART
        // TODO: ONTOLOGY IRI CHECK IN RESOLVER
        // TODO: ONTOLOGY OWL IMPORT INJECTION
        // TODO: Backup the earlier folder structure and ignore that for everything. (Easy to implement)
        // TODO: Git ssh, fetch with cli?

        // Always update the index with some plow commands.
        Ok(())

    } else {
        std::process::exit(0x00);
    }
}
