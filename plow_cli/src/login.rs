use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct CredentialsFile<'cred> {
    #[serde(borrow)]
    pub registry: Registry<'cred>,
}

impl<'cred> CredentialsFile<'cred> {
    /// Returns the token for the registry.
    pub const fn with_token(token: &'cred str) -> Self {
        CredentialsFile {
            registry: Registry::new(token),
        }
    }
}

/// Registry table in credentials file (toml).
#[derive(Serialize, Debug, Deserialize)]
pub struct Registry<'reg> {
    token: &'reg str,
}

impl<'reg> Registry<'reg> {
    /// Returns the token for the registry.
    pub const fn new(token: &'reg str) -> Self {
        Registry { token }
    }
}

pub fn save_credentials_replace_existing(token: &str) -> Result<()> {
    let credentials_contents =
        toml::to_string::<CredentialsFile>(&CredentialsFile::with_token(token))?;
    let config_dir = super::config::get_config_dir()?;
    std::fs::write(config_dir.join("credentials"), credentials_contents)?;
    Ok(())
}

pub fn get_api_token() -> Result<String> {
    let config_dir = super::config::get_config_dir()?;
    let credentials_file_contents = std::fs::read_to_string(config_dir.join("credentials"))?;
    let credentials = toml::from_str::<CredentialsFile>(&credentials_file_contents)?;
    Ok(credentials.registry.token.to_owned())
}
