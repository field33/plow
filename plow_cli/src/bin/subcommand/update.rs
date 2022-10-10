use std::collections::HashMap;

use crate::{error::CliError, error::FieldAccessError::*, error::WorkspaceInitializationError::*};
use plow::manifest::FieldManifest;

use clap::ArgMatches;
use clap::{App, Command};

use plow_package_management::lock::{LockFile, PackageInLockFile};

use plow_package_management::registry::Registry;

use crate::config::PlowConfig;

use crate::feedback::{general_update_success, Feedback};
use crate::resolve::resolve;

use super::init::workspace::fields::FieldsDirectory;

pub struct SuccessfulUpdate;
impl Feedback for SuccessfulUpdate {
    fn feedback(&self) {
        general_update_success();
    }
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("update")
        .about("Updates the registry index, caches dependencies and updates the lock file.")
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches, config: &PlowConfig) -> Box<dyn Feedback + 'static> {
    match run_command_flow(sub_matches, config) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

pub fn run_command_flow(_: &ArgMatches, config: &PlowConfig) -> Result<impl Feedback, CliError> {
    let workspace_root = config.working_dir.fail_if_not_under_a_workspace()?;
    let mut fields_dir = FieldsDirectory::fill_from_root(&workspace_root.join("fields"))?;
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
        let root_field_manifest = FieldManifest::new(&root_field_contents).map_err(|_| {
            CliError::from(FailedToReadFieldManifest {
                field_path: child.as_path().to_string(),
            })
        })?;

        #[allow(clippy::unwrap_used)]
        let root_field_name = root_field_manifest.full_name();
        let root_dep_names = root_field_manifest.dependencies();

        if let Ok(Some(fresh_lock_file)) = resolve(
            config,
            &root_field_contents,
            &root_field_manifest,
            false,
            &registry as &dyn Registry,
        ) {
            // Unwrap is fine here we've linted the field before.
            #[allow(clippy::unwrap_used)]
            let root_as_index = root_field_manifest.as_index();
            // Check for duplicate names
            if collection.get(&root_field_name).is_some() {
                return Err(CliError::from(DuplicateFieldInWorkspace(root_field_name)));
            }
            collection.insert(
                root_field_name.clone(),
                (
                    PackageInLockFile {
                        name: root_as_index.name(),
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

    Ok(SuccessfulUpdate)
}
