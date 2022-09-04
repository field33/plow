use camino::{Utf8Path, Utf8PathBuf};
use plow_linter::lints::field_manifest_lints;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;

use crate::config::PlowConfig;
use crate::error::CliError;
use crate::error::WorkspaceInitializationError::*;
use crate::manifest::FieldManifest;
use crate::subcommand::lint::lint_file_fail_on_failure;

pub struct FieldsDirectory {
    pub children: Vec<Utf8PathBuf>,
    pub path: Utf8PathBuf,
}

impl FieldsDirectory {
    pub fn empty_with_path(path: &Utf8Path) -> Self {
        Self {
            children: vec![],
            path: path.to_path_buf(),
        }
    }
    pub fn exists_in_filesystem(&self) -> bool {
        self.path.exists()
    }

    pub fn lint_all_children(&self) -> Option<(Vec<String>, CliError)> {
        // Lint all fields in the directory and collect failures if there are some.
        let failed_field_paths_on_linting = self
            .children
            .par_iter()
            .filter_map(|path| {
                // TODO: Field manifest or all lints?
                if let Err(err) = lint_file_fail_on_failure(path.as_ref(), field_manifest_lints()) {
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

    pub fn write_with_children(&self) -> Result<(), CliError> {
        std::fs::create_dir_all(&self.path)
            .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

        for child in &self.children {
            // We're safe here, we've linted before.
            if let Ok(full_name) = FieldManifest::quick_extract_field_full_name(&child) {
                let full_name: Vec<&str> = full_name.split('/').collect();
                let namespace = full_name[0];
                let name = full_name[1];

                std::fs::create_dir_all(self.path.join(&namespace))
                    .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

                std::fs::create_dir_all(self.path.join(&namespace).join(&name))
                    .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;

                #[allow(clippy::unwrap_used)]
                let new_field_destination = self
                    .path
                    .join(&namespace)
                    .join(&name)
                    // Checked before
                    .join(&child.file_name().unwrap());

                std::fs::copy(child, &new_field_destination)
                    .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;
            }
        }
        Ok(())
    }

    // Explodes the fields directory back to the workspace root.
    pub fn explode(path_to_fields_dir: &Utf8Path, config: &PlowConfig) -> Result<(), CliError> {
        if path_to_fields_dir.exists() {
            let fields = crate::utils::list_files(path_to_fields_dir, "ttl")
                .map_err(|err| FailedToReadFieldsDirectory(err.to_string()))?;
            for field in fields {
                std::fs::copy(field, &config.working_dir.path)
                    .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?;
            }
            std::fs::remove_dir_all(&path_to_fields_dir)
                .map_err(|err| FailedToRemoveFieldsDirectory(err.to_string()))?;
            return Ok(());
        }
        Err(FailedToReadFieldsDirectory("fields directory does not exist".to_string()).into())
    }

    pub fn write_empty(&self) -> Result<(), CliError> {
        Ok(std::fs::create_dir_all(&self.path)
            .map_err(|err| FailedToCreateFieldsDirectory(err.to_string()))?)
    }
}
