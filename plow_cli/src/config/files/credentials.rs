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
    pub token: &'reg str,
}

impl<'reg> Registry<'reg> {
    /// Returns the token for the registry.
    pub const fn new(token: &'reg str) -> Self {
        Registry { token }
    }
}
