use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{
    error::CliError, error::WorkspaceInitializationError::*, manifest::FieldManifest,
    subcommand::init::workspace::fields::FieldsDirectory,
};
/// Serialized example:
///
/// ```toml
/// # Plow.toml
///
/// [workspace]
/// members = ["path to field","another", ..]
///
/// ```
#[derive(Serialize, Debug, Deserialize, Default)]
pub struct WorkspaceManifestFile {
    pub workspace: Option<Workspace>,
    #[serde(skip_serializing, skip_deserializing)]
    pub path: Utf8PathBuf,
}

impl WorkspaceManifestFile {
    pub fn empty_with_path(path: &Utf8Path) -> Self {
        Self {
            workspace: None,
            path: path.to_path_buf(),
        }
    }

    pub fn get_member_paths(&self) -> Vec<Utf8PathBuf> {
        self.workspace.as_ref().map_or_else(Vec::new, |workspace| {
            workspace.member_map.values().cloned().collect()
        })
    }

    pub fn get_member_names(&self) -> Vec<String> {
        self.workspace.as_ref().map_or_else(Vec::new, |workspace| {
            workspace.member_map.keys().cloned().collect()
        })
    }

    pub fn get_member_path_by_member_name(&self, name: &str) -> Option<Utf8PathBuf> {
        if let Some(ref workspace) = self.workspace {
            return workspace.member_map.get(name).cloned();
        }
        None
    }

    pub fn get_member_name_by_member_path(&self, path: &Utf8Path) -> Option<String> {
        if let Some(ref workspace) = self.workspace {
            return workspace
                .member_map
                .iter()
                .find_map(|(key, val)| {
                    if val.as_path() == path {
                        return Some(key);
                    }
                    None
                })
                .cloned();
        }
        None
    }

    pub fn from_file(path: &Utf8Path) -> Result<Self, CliError> {
        let contents = std::fs::read(path)
            .map_err(|err| FailedToReadWorkspaceManifestFile(err.to_string()))?;
        let mut workspace_manifest_file = toml::from_slice::<Self>(&contents)
            .map_err(|err| FailedToReadWorkspaceManifestFile(err.to_string()))?;
        workspace_manifest_file.path = path.to_path_buf();
        Ok(workspace_manifest_file)
    }

    pub fn fetch(&mut self) -> Result<(), CliError> {
        let contents = std::fs::read(&self.path)
            .map_err(|err| FailedToReadWorkspaceManifestFile(err.to_string()))?;
        let updated_workspace_manifest_file = toml::from_slice::<Self>(&contents)
            .map_err(|err| FailedToReadWorkspaceManifestFile(err.to_string()))?;
        self.workspace = updated_workspace_manifest_file.workspace;
        Ok(())
    }

    pub fn write(&self) -> Result<(), CliError> {
        let contents = toml::to_vec(&self)
            .map_err(|err| FailedToWriteWorkspaceManifestFile(err.to_string()))?;
        std::fs::write(&self.path, contents)
            .map_err(|err| FailedToWriteWorkspaceManifestFile(err.to_string()))?;
        Ok(())
    }
}

#[derive(Serialize, Debug, Deserialize, Default)]
pub struct Workspace {
    pub members: Option<Vec<String>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub member_map: std::collections::HashMap<String, Utf8PathBuf>,
}

// TODO:
impl From<&FieldsDirectory> for Workspace {
    fn from(fields_dir: &FieldsDirectory) -> Self {
        let mut member_map = std::collections::HashMap::new();
        let mut members = vec![];
        for path in &fields_dir.children {
            if let Ok(name) = FieldManifest::quick_extract_field_full_name(&path.as_path()) {
                member_map.insert(name.clone(), path.as_path_buf());
                members.push(name);
                continue;
            }
            // TODO:
            // Give some feedback to the user that names couldn't be extracted and workspace does not recognize these fields.
            // Do not fail the command.
        }
        Self {
            members: Some(members),
            member_map,
        }
    }
}

impl From<&FieldsDirectory> for WorkspaceManifestFile {
    fn from(fields_dir: &FieldsDirectory) -> Self {
        let workspace = Some(Workspace::from(fields_dir));
        let path = fields_dir
            .path
            .parent()
            // Pretty handle failure here converting to try from.
            .expect("Fields directory is in os root, probably you didn't want this to happen.")
            .to_path_buf()
            .join("Plow.toml");
        Self { workspace, path }
    }
}
