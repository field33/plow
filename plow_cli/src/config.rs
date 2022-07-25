use anyhow::{anyhow, bail, Result};

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
