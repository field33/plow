use crate::{config::WorkingDirectory, error::CliError, error::ConfigError::*};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

/// Serialized example:
///
/// ```toml
/// # config.toml
///
/// [plow]
/// home = "~/.plow"
///
/// [registry]
/// index = "https://api.plow.pm"
/// token = "an auth token to override the one in the credentials file"
///
/// [net]
/// # Stops calls to the remote registry.
/// offline = true
/// ```
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct WorkspaceConfigFile {
    pub plow: Option<Plow>,
    pub registry: Option<Registry>,
    pub net: Option<Net>,
    #[serde(skip_serializing, skip_deserializing)]
    pub path: Utf8PathBuf,
}

impl WorkspaceConfigFile {
    pub fn empty_with_path(path: &Utf8Path) -> Self {
        Self {
            plow: None,
            registry: None,
            net: None,
            path: path.to_path_buf(),
        }
    }

    pub fn set_plow_home(&mut self, path: &Utf8Path) {
        if let Some(ref mut plow) = self.plow {
            plow.home = Some(path.to_string());
        } else {
            self.plow = Some(Plow {
                home: Some(path.to_string()),
            });
        }
    }
    pub fn from_file(path: &Utf8Path) -> Result<Self, CliError> {
        let contents =
            std::fs::read(path).map_err(|err| FailedToReadWorkspaceConfigFile(err.to_string()))?;
        let mut workspace_config_file = toml::from_slice::<Self>(&contents)
            .map_err(|err| FailedToReadWorkspaceConfigFile(err.to_string()))?;
        workspace_config_file.path = path.to_path_buf();
        Ok(workspace_config_file)
    }
    pub fn fetch(&self) -> Result<Self, CliError> {
        let contents = std::fs::read(&self.path)
            .map_err(|err| FailedToReadWorkspaceConfigFile(err.to_string()))?;
        let updated_workspace_config_file = toml::from_slice::<Self>(&contents)
            .map_err(|err| FailedToReadWorkspaceConfigFile(err.to_string()))?;
        Ok(updated_workspace_config_file)
    }
    pub fn write(&self) -> Result<(), CliError> {
        let contents =
            toml::to_vec(&self).map_err(|err| FailedToWriteWorkspaceConfigFile(err.to_string()))?;
        std::fs::write(&self.path, contents)
            .map_err(|err| FailedToWriteWorkspaceConfigFile(err.to_string()))?;
        Ok(())
    }
    pub fn create_in_working_dir(working_dir: &WorkingDirectory) -> Result<Self, CliError> {
        working_dir.fail_if_not_workspace()?;
        let workspace_config_dir_path = working_dir.path.join(".plow");
        let workspace_config_file_path = workspace_config_dir_path.join("config.toml");

        if !workspace_config_dir_path.exists() {
            std::fs::create_dir_all(&workspace_config_dir_path)
                .map_err(|err| FailedToCreateWorkspaceConfigDirectory(err.to_string()))?;
        }

        if workspace_config_file_path.exists() {
            Self::from_file(&workspace_config_file_path)
        } else {
            let mut workspace_config_file = Self::empty_with_path(&workspace_config_file_path);
            workspace_config_file.path = workspace_config_file_path;
            workspace_config_file.write()?;
            Ok(workspace_config_file)
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Plow {
    pub home: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Registry {
    pub index: Option<String>,
    pub token: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Net {
    pub offline: Option<bool>,
}
