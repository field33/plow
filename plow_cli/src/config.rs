pub mod files;

use crate::error::CliError;
use crate::error::ConfigError::*;
use crate::error::LoginError::*;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use std::str::FromStr;

use self::files::credentials::CredentialsFile;
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
pub struct PlowDirs {
    plow_home: Utf8PathBuf,
    user_home: Option<Utf8PathBuf>,
    credentials_path: Utf8PathBuf,
    registry_dir: Utf8PathBuf,
    field_cache_dir: Utf8PathBuf,
    index_dir: Utf8PathBuf,
    index_cache_dir: Utf8PathBuf,
    working_dir: Utf8PathBuf,
}

impl PlowDirs {
    pub fn home(&self) -> &Utf8Path {
        &self.plow_home
    }
    pub fn user_home(&self) -> Option<&Utf8Path> {
        self.user_home.as_deref()
    }
    pub fn credentials_path(&self) -> &Utf8Path {
        &self.credentials_path
    }
    pub fn registry(&self) -> &Utf8Path {
        &self.registry_dir
    }
    pub fn field_cache(&self) -> &Utf8Path {
        &self.field_cache_dir
    }
    pub fn index(&self) -> &Utf8Path {
        &self.index_dir
    }
    pub fn index_cache(&self) -> &Utf8Path {
        &self.index_cache_dir
    }
    pub fn working_dir(&self) -> &Utf8Path {
        &self.working_dir
    }
}

#[derive(Debug)]
pub struct PlowFiles {
    workspace_config: Option<WorkspaceConfigFile>,
}

impl PlowFiles {
    pub fn workspace_config(&self) -> Option<&WorkspaceConfigFile> {
        self.workspace_config.as_ref()
    }
}

#[derive(Debug)]
pub struct PlowOptions {
    registry_url: Option<String>,
}

impl PlowOptions {
    pub fn registry_url(&self) -> Option<&str> {
        self.registry_url.as_deref()
    }
}

#[derive(Debug)]
pub struct PlowConfig {
    dirs: PlowDirs,
    files: PlowFiles,
    options: PlowOptions,
    workspace: Option<PlowWorkspaceConfig>,
}

impl PlowConfig {
    pub fn user_home(&self) -> Option<&Utf8Path> {
        self.dirs.user_home()
    }

    pub fn field_cache_dir(&self) -> &Utf8Path {
        self.dirs.field_cache()
    }

    pub fn index_dir(&self) -> &Utf8Path {
        self.dirs.index()
    }

    pub fn working_dir(&self) -> &Utf8Path {
        self.dirs.working_dir()
    }

    pub fn registry_url(&self) -> Option<&str> {
        self.options.registry_url()
    }

    pub fn new(
        dirs: PlowDirs,
        files: PlowFiles,
        options: PlowOptions,
        workspace: Option<PlowWorkspaceConfig>,
    ) -> Self {
        Self {
            dirs,
            files,
            options,
            workspace,
        }
    }

    pub fn workspace_config(&self) -> Option<&PlowWorkspaceConfig> {
        self.workspace.as_ref()
    }

    pub fn workspace_root(&self) -> Option<&Utf8Path> {
        if let Some(ws) = self.workspace {
            Some(ws.workspace_root())
        } else {
            None
        }
    }

    pub fn get_registry_url(&self) -> Result<&str, CliError> {
        // Check if user provided any with --registry
        if let Some(registry_url) = self.registry_url() {
            return Ok(registry_url);
        }

        // Check for .plow folder in the workspace for config.toml which might have a registry url.
        if let Some(workspace_config) = self.workspace_config() {
            let workspace_config_file = workspace_config.fetch()?;
            if let Some(registry) = workspace_config_file.registry {
                if let Some(url) = registry.index {
                    return Ok(&url);
                }
            }
        }

        // Fall back to default registry url.
        Ok(DEFAULT_REGISTRY_URL)
    }

    pub fn api_token(&self) -> Result<String, CliError> {
        // Check for .plow folder in the workspace for config.toml which might have a token to override.
        if let Some(workspace_config) = self.workspace_config() {
            let workspace_config_file = workspace_config.fetch()?;

            // TODO: This place should modify if we wish to support multiple registries.
            if let Some(registry) = workspace_config_file.registry {
                if let Some(token) = registry.token {
                    return Ok(token);
                }
            }
        }

        let credentials_file_contents = std::fs::read_to_string(&self.dirs.credentials_path)
            .map_err(|_| FailedToReadCredentialsFile)?;
        let credentials = toml::from_str::<CredentialsFile>(&credentials_file_contents)
            .map_err(|_| FailedToReadCredentialsFile)?;

        Ok(credentials.registry.token.to_owned())
    }

    pub fn is_workspace(&self) -> bool {
        self.workspace.is_some()
    }
    pub fn fail_if_not_under_a_workspace(&self) -> Result<(), CliError> {
        if self.workspace.is_none() {
            return Err(CliError::from(DirectoryNotWorkspace));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PlowWorkspaceConfig {
    pub root: Utf8PathBuf,
    pub file: WorkspaceConfigFile,
}

impl PlowWorkspaceConfig {
    pub fn workspace_root(&self) -> &Utf8Path {
        &self.root
    }

    pub fn fetch(&self) -> Result<WorkspaceConfigFile, CliError> {
        self.file.fetch()
    }

    pub fn workspace_config_file(&self) -> &WorkspaceConfigFile {
        &self.file
    }

    // TODO: Revisit
    pub fn create_config_file(&mut self) -> Result<(), CliError> {
        let workspace_config_dir_path = self.root.join(".plow");
        let workspace_config_file_path = workspace_config_dir_path.join("config.toml");

        if !workspace_config_dir_path.exists() {
            std::fs::create_dir_all(&workspace_config_dir_path)
                .map_err(|err| FailedToCreateWorkspaceConfigDirectory(err.to_string()))?;
        }

        if workspace_config_file_path.exists() {
            self.file = WorkspaceConfigFile::from_file(&workspace_config_file_path)?;
            Ok(())
        } else {
            let mut workspace_config_file =
                WorkspaceConfigFile::empty_with_path(&workspace_config_file_path);
            workspace_config_file.path = workspace_config_file_path;
            workspace_config_file.write()?;
            self.file = workspace_config_file;
            Ok(())
        }
    }
}

impl TryFrom<&Utf8Path> for PlowWorkspaceConfig {
    type Error = anyhow::Error;

    fn try_from(path_possibly_under_workspace: &Utf8Path) -> Result<Self, Self::Error> {
        if let Some(workspace_root) = find_workspace_root(path_possibly_under_workspace) {
            if workspace_root.join("Plow.toml").exists() {
                let workspace_config_file =
                    WorkspaceConfigFile::from_file(&workspace_root.join("Plow.toml"))?;
                let workspace_config = workspace_config_file.fetch()?;
                return Ok(Self {
                    root: workspace_root.to_path_buf(),
                    file: workspace_config_file,
                });
            }
            return Err(anyhow::anyhow!("Workspace is corrupt"));
        }
        Err(anyhow::anyhow!("Failed to find workspace root"))
    }
}

// TODO: Currently Plow does not support multiple registries.
// There is one central remote registry which is owned by Field33.
// Although there are plans to expand support for custom registries in the future.
#[allow(clippy::unwrap_used)]
#[allow(clippy::missing_panics_doc)]
pub fn configure(
    custom_path: Option<Utf8PathBuf>,
    registry_url_override: Option<String>,
) -> Result<PlowConfig, CliError> {
    let working_dir = Utf8PathBuf::from_path_buf(
        std::env::current_dir().map_err(|err| FailedToGetWorkingDirectory(err.to_string()))?,
    )
    .map_err(|_| {
        FailedToGetWorkingDirectory(
            "The path to the home directory is not UTF8 encoded.".to_owned(),
        )
    })?;

    let mut workspace_config_file = None;

    let mut user_home: Option<Utf8PathBuf> = None;
    let plow_home = if let Some(custom_path) = custom_path {
        // let mut new_workspace_config_file =
        //     WorkspaceConfigFile::create_in_working_dir(&working_dir)?;
        // new_workspace_config_file.set_plow_home(&custom_path);
        // new_workspace_config_file.write()?;
        // workspace_config_file = Some(new_workspace_config_file);
        // custom_path.join(".plow")
        todo!()
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
        user_home = Some(homedir.clone());
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

    let workspace = if let Some(root) = find_workspace_root(&working_dir) {
        PlowWorkspaceConfig::try_from(root).ok()
    } else {
        None
    };

    Ok(PlowConfig::new(
        PlowDirs {
            user_home,
            plow_home,
            credentials_path: credentials_file,
            registry_dir,
            field_cache_dir,
            index_dir,
            index_cache_dir,
            working_dir,
        },
        PlowFiles {
            workspace_config: workspace_config_file,
        },
        PlowOptions {
            registry_url: registry_url_override,
        },
        workspace,
    ))
}

fn find_workspace_root(path: &Utf8Path) -> Option<&Utf8Path> {
    if path.join("Plow.toml").exists() {
        return Some(path);
    }
    if let Some(parent) = path.parent() {
        return find_workspace_root(parent);
    }
    None
}

pub fn remove_configuration_directory_if_exists(config: &PlowConfig) -> Result<(), CliError> {
    if !config.dirs.home().exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(config.dirs.home())
        .map_err(|err| FailedToRemoveConfigDirectory(err.to_string()))?;
    Ok(())
}
