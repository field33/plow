use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

/// Struct for a midway fetch from database it gets  transformed to a `FieldSummary` later.
#[derive(Debug, Clone, FromQueryResult, Eq, PartialOrd, Ord)]
pub struct FieldAndCategories {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub submitted_at: String,
    pub submitter_id: String,
    pub submitter_name: String,
    pub category_name: String,
    pub category_description: String,
}

impl PartialEq for FieldAndCategories {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, FromQueryResult, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldHash {
    pub field_hash: String,
}

#[derive(Debug, Clone, FromQueryResult, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamespaceAndName {
    pub namespace: String,
    pub name: String,
}

// TODO: Search terms might be expanded later.
#[derive(Deserialize, Serialize)]
pub struct FieldSearchForm {
    pub name: String,
}
