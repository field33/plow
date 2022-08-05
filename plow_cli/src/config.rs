use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Default)]
pub struct PlowConfigFile<'cred> {
    pub workspace: Workspace,
    #[serde(borrow)]
    pub registry: Registry<'cred>,
}

impl PlowConfigFile<'_> {
    /// Returns the token for the registry.
    pub fn with_workspace(workspace: &Workspace) -> Self {
        PlowConfigFile {
            registry: Registry::default(),
            workspace: workspace.clone(),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Default, Clone)]
pub struct Workspace {
    pub members: Vec<String>,
}

impl From<Vec<std::path::PathBuf>> for Workspace {
    fn from(paths: Vec<std::path::PathBuf>) -> Self {
        Workspace {
            members: paths
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect(),
        }
    }
}
impl From<Vec<camino::Utf8PathBuf>> for Workspace {
    fn from(paths: Vec<camino::Utf8PathBuf>) -> Self {
        Workspace {
            members: paths.iter().map(|path| path.to_string()).collect(),
        }
    }
}

/// Registry table in credentials file (toml).
#[derive(Serialize, Debug, Deserialize)]
pub struct Registry<'reg> {
    url: &'reg str,
}

impl<'reg> Registry<'reg> {
    /// Returns the token for the registry.
    pub const fn new(url: &'reg str) -> Self {
        Registry { url }
    }
}

impl Default for Registry<'_> {
    fn default() -> Self {
        Registry {
            url: "https://staging-api.plow.pm",
        }
    }
}

// For more information: <http://www.brynosaurus.com/cachedir/>
const CACHE_DIRECTORY_TAG_FILE_NAME: &str = "CACHEDIR.TAG";
const CACHE_DIRECTORY_TAG_FILE_CONTENTS: &str = r#"
Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by plow.
# For information about cache directory tags, see:
#	http://www.brynosaurus.com/cachedir/
"#;

pub fn get_config_dir() -> Result<std::path::PathBuf> {
    let homedir =
        dirs::home_dir().ok_or_else(|| anyhow!("User home directory could not be found."))?;
    Ok(homedir.join(".plow"))
}

pub fn get_registry_url() -> Result<String> {
    let config_file_path = camino::Utf8PathBuf::from("./Plow.toml");
    let config_file_contents = std::fs::read_to_string(&config_file_path)?;
    let config_file = toml::from_str::<PlowConfigFile>(&config_file_contents)?;
    Ok(config_file.registry.url.to_owned())
}

// TODO: Revisit initial structure
// config.toml file?
// credentials.toml file instead of credentials?
pub fn create_configuration_directory_if_not_exists() -> Result<camino::Utf8PathBuf> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        return Ok(config_dir.to_string_lossy().as_ref().into());
    }

    std::fs::create_dir_all(&config_dir)?;
    std::fs::write(
        config_dir.join("credentials"),
        "# `plow login <your-api-token>` will store your api token in this file.\n",
    )?;

    let registry_dir = config_dir.join("registry");
    std::fs::create_dir_all(&registry_dir)?;
    std::fs::write(
        config_dir
            .join("registry")
            .join(CACHE_DIRECTORY_TAG_FILE_NAME),
        CACHE_DIRECTORY_TAG_FILE_CONTENTS,
    )?;
    std::fs::create_dir_all(registry_dir.join("artifact_cache"))?;

    let index_dir = registry_dir.join("index");
    std::fs::create_dir_all(&index_dir)?;
    std::fs::create_dir_all(index_dir.join(".cache"))?;
    std::fs::write(index_dir.join(".last_updated"), "")?;
    std::fs::create_dir_all(index_dir.join(".cache").join("public"))?;
    std::fs::create_dir_all(index_dir.join(".cache").join("private"))?;

    Ok(config_dir.to_string_lossy().as_ref().into())
}

pub fn remove_configuration_directory_if_exists() -> Result<()> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&config_dir)?;
    Ok(())
}

pub fn clean_configuration_directory() -> Result<camino::Utf8PathBuf> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        return create_configuration_directory_if_not_exists();
    }
    remove_configuration_directory_if_exists()?;
    create_configuration_directory_if_not_exists()
}
