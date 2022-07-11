use serde::{Deserialize, Serialize};

/// A type which represents a category to be returned in a response.
///
/// Categories are unique in the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Category {
    pub name: String,
    pub description: String,
}
impl Category {
    pub const fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

/// A type which holds meta data about a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTokenSummary {
    pub id: i32,
    pub name: String,
    pub expires: i32,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// A type which represents a field but in a summarized way to be returned in a response.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct FieldSummary {
    /// Field hash which is produced by hashing the namespace, name and version like the following:
    ///
    /// "<namespace>/<name> <version>"
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub version: String,

    pub categories: Vec<Category>,

    pub submitted_at: String,
    /// Corresponds to the auth0 id of the user.
    pub submitter_id: String,
    /// The readable name of the user.
    pub submitter_name: String,
}

#[allow(clippy::too_many_arguments)]
impl FieldSummary {
    pub const fn new(
        id: String,
        namespace: String,
        name: String,
        version: String,
        categories: Vec<Category>,
        submitted_at: String,
        submitter_id: String,
        submitter_name: String,
    ) -> Self {
        Self {
            id,
            namespace,
            name,
            version,
            categories,
            submitted_at,
            submitter_id,
            submitter_name,
        }
    }
}

impl PartialEq for FieldSummary {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
