pub mod files;

use crate::error::CliError;
use crate::error::ConfigError::*;
use crate::error::LoginError::*;
use crate::subcommand::login::CredentialsFile;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use std::str::FromStr;

use self::files::workspace_config::WorkspaceConfigFile;

// For more information: <http://www.brynosaurus.com/cachedir/>
pub const CACHE_DIRECTORY_TAG_FILE_NAME: &str = "CACHEDIR.TAG";
pub const CACHE_DIRECTORY_TAG_FILE_CONTENTS: &str = r#"
Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by plow.
# For information about cache directory tags, see:
#	http://www.brynosaurus.com/cachedir/
"#;

// TODO: Change back
pub const DEFAULT_REGISTRY_URL: &str = "https://api.plow.pm";

#[derive(Debug)]
pub struct PlowConfig {
    pub plow_home: Utf8PathBuf,
    pub credentials_path: Utf8PathBuf,
    pub registry_dir: Utf8PathBuf,
    pub field_cache_dir: Utf8PathBuf,
    pub index_dir: Utf8PathBuf,
    pub index_cache_dir: Utf8PathBuf,
    pub working_dir: WorkingDirectory,
    workspace_config_file: Option<WorkspaceConfigFile>,
    // Fill this if it is provided with a command.
    registry_url: Option<String>,
}

impl PlowConfig {
    fn find_workspace_root(&self, path: &Utf8Path) -> Result<Utf8PathBuf, CliError> {
        if path.join("Plow.toml").exists() {
            return Ok(path.to_path_buf());
        }
        if let Some(parent) = path.parent() {
            return self.find_workspace_root(parent);
        }
        Err(CliError::from(FailedToFindWorkspaceRoot))
    }

    pub fn get_workspace_root(&self) -> Result<Utf8PathBuf, CliError> {
        self.find_workspace_root(&self.working_dir.path)
    }

    pub fn get_registry_url(&self) -> Result<String, CliError> {
        // Check if user provided any with --registry
        if let Some(ref registry_url) = self.registry_url {
            return Ok(registry_url.clone());
        }

        // Check for .plow folder in the workspace for config.toml which might have a registry url.
        if let Some(ref workspace_config_file) = self.workspace_config_file {
            let workspace_config_file = workspace_config_file.fetch()?;
            if let Some(registry) = workspace_config_file.registry {
                if let Some(url) = registry.index {
                    return Ok(url);
                }
            }
        }

        // Fall back to default registry url.
        Ok(DEFAULT_REGISTRY_URL.to_owned())
    }

    pub fn get_saved_api_token(&self) -> Result<String, CliError> {
        // Check for .plow folder in the workspace for config.toml which might have a token to override.
        if let Some(ref workspace_config_file) = self.workspace_config_file {
            let workspace_config_file = workspace_config_file.fetch()?;
            // TODO: This place should modify if we wish to support multiple registries.
            if let Some(registry) = workspace_config_file.registry {
                if let Some(token) = registry.token {
                    return Ok(token);
                }
            }
        }

        let credentials_file_contents = std::fs::read_to_string(&self.credentials_path)
            .map_err(|_| FailedToReadCredentialsFile)?;
        let credentials = toml::from_str::<CredentialsFile>(&credentials_file_contents)
            .map_err(|_| FailedToReadCredentialsFile)?;

        Ok(credentials.registry.token.to_owned())
    }
}

#[derive(Debug)]
pub struct WorkingDirectory {
    pub path: Utf8PathBuf,
}

impl WorkingDirectory {
    pub fn is_workspace(&self) -> bool {
        self.path.join("Plow.toml").exists()
    }
    pub fn fail_if_not_workspace(&self) -> Result<(), CliError> {
        if !self.path.join("Plow.toml").exists() {
            return Err(CliError::from(DirectoryNotWorkspace));
        }
        Ok(())
    }
    pub fn fail_if_not_under_a_workspace(&self) -> Result<Utf8PathBuf, CliError> {
        self.get_workspace_root()
            .map_or_else(|_| Err(CliError::from(DirectoryNotWorkspace)), Ok)
    }

    fn find_workspace_root(&self, path: &Utf8Path) -> Result<Utf8PathBuf, CliError> {
        if path.join("Plow.toml").exists() {
            return Ok(path.to_path_buf());
        }
        if let Some(parent) = path.parent() {
            return self.find_workspace_root(parent);
        }
        Err(CliError::from(FailedToFindWorkspaceRoot))
    }

    pub fn get_workspace_root(&self) -> Result<Utf8PathBuf, CliError> {
        self.find_workspace_root(&self.path)
    }
}

impl From<Utf8PathBuf> for WorkingDirectory {
    fn from(path: Utf8PathBuf) -> Self {
        Self { path }
    }
}

// TODO: Currently Plow does not support multiple registries.
// There is one central remote registry which is owned by Field33.
// Although there are plans to expand support for custom registries in the future.
#[allow(clippy::unwrap_used)]
#[allow(clippy::missing_panics_doc)]
pub fn configure(
    custom_path: Option<Utf8PathBuf>,
    registry_url: Option<String>,
) -> Result<PlowConfig, CliError> {
    let working_dir = WorkingDirectory::from(
        Utf8PathBuf::from_path_buf(
            std::env::current_dir().map_err(|err| FailedToGetWorkingDirectory(err.to_string()))?,
        )
        .map_err(|_| {
            FailedToGetWorkingDirectory(
                "The path to the home directory is not UTF8 encoded.".to_owned(),
            )
        })?,
    );

    let mut workspace_config_file = None;

    let plow_home = if let Some(custom_path) = custom_path {
        let mut new_workspace_config_file =
            WorkspaceConfigFile::create_in_working_dir(&working_dir)?;
        new_workspace_config_file.set_plow_home(&custom_path);
        new_workspace_config_file.write()?;
        workspace_config_file = Some(new_workspace_config_file);
        custom_path.join(".plow")
    } else {
        let homedir = dirs::home_dir().ok_or_else(|| {
            FailedToReadOrCreateConfigDirectory(
                Utf8PathBuf::from_str("~/.plow").unwrap().into(),
                "User home directory could not be found or read.".to_owned(),
            )
        })?;
        let homedir = Utf8PathBuf::from_path_buf(homedir).map_err(|non_utf8_path_buf| {
            FailedToReadOrCreateConfigDirectory(
                non_utf8_path_buf.to_string_lossy().into(),
                "The path to the home directory is not UTF8 encoded.".to_owned(),
            )
        })?;
        homedir.join(".plow")
    };

    // Create if not there already
    if !plow_home.exists() {
        std::fs::create_dir_all(&plow_home).map_err(|err| {
            FailedToReadOrCreateConfigDirectory(plow_home.clone().into(), err.to_string())
        })?;
    }

    let credentials_file = plow_home.join("credentials.toml");
    if !credentials_file.exists() {
        std::fs::write(
            plow_home.join("credentials.toml"),
            "# `plow login <your-api-token>` will store your api token in this file.\n",
        )
        .map_err(|err| FailedToWriteToConfigDirectory(err.to_string()))?;
    }

    let registry_dir = plow_home.join("registry");
    if !registry_dir.exists() {
        std::fs::create_dir_all(&registry_dir)
            .map_err(|err| FailedToWriteToConfigDirectory(err.to_string()))?;
    }

    let cache_dir_tag = registry_dir.join(CACHE_DIRECTORY_TAG_FILE_NAME);
    if !cache_dir_tag.exists() {
        std::fs::write(cache_dir_tag, CACHE_DIRECTORY_TAG_FILE_CONTENTS)
            .map_err(|err| FailedToWriteToConfigDirectory(err.to_string()))?;
    }

    let field_cache_dir = registry_dir.join("cache");
    if !field_cache_dir.exists() {
        std::fs::create_dir_all(registry_dir.join("cache"))
            .map_err(|err| FailedToWriteToConfigDirectory(err.to_string()))?;
    }

    let index_dir = registry_dir.join("index");
    if !index_dir.exists() {
        std::fs::create_dir_all(&index_dir)
            .map_err(|err| FailedToWriteToConfigDirectory(err.to_string()))?;
    }

    let index_cache_dir = index_dir.join(".cache");
    if !index_cache_dir.exists() {
        std::fs::create_dir_all(&index_cache_dir)
            .map_err(|err| FailedToWriteToConfigDirectory(err.to_string()))?;
    }

    Ok(PlowConfig {
        plow_home,
        credentials_path: credentials_file,
        registry_dir,
        field_cache_dir,
        index_dir,
        index_cache_dir,
        working_dir,
        workspace_config_file,
        registry_url,
    })
}

pub fn remove_configuration_directory_if_exists(config: &PlowConfig) -> Result<(), CliError> {
    if !config.plow_home.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&config.plow_home)
        .map_err(|err| FailedToRemoveConfigDirectory(err.to_string()))?;
    Ok(())
}
