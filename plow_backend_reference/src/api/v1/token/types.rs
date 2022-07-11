use serde::{Deserialize, Serialize};

/// A structure to model the form for creating a new token.
#[derive(Deserialize, Serialize)]
pub struct TokenGenerationForm {
    pub name: String,
}

/// A structure to represent a newly generated client secret.
pub struct EncryptedClientSecretGenerationResult {
    pub basic_auth_token: String,
    pub encrypted_client_secret: String,
}
