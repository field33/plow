#![allow(clippy::use_self)]

use anyhow::bail;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::feedback::{submission_failed, submission_remote_linting_failed, Feedback};

/// `status` field of the response.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]

pub enum StatusInfo {
    #[serde(rename(serialize = "success", deserialize = "success"))]
    Success,
    #[serde(rename(serialize = "fail", deserialize = "fail"))]
    Failure,
    #[serde(rename(serialize = "error", deserialize = "error"))]
    Error,
}

impl TryFrom<&str> for StatusInfo {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "success" => Ok(Self::Success),
            "fail" => Ok(Self::Failure),
            "error" => Ok(Self::Error),
            s => bail!("Invalid status text: {}", s),
        }
    }
}

/// A response with success status.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend#success) spec
#[derive(Debug, Serialize, Deserialize)]
pub struct Success {
    pub status: StatusInfo,
    pub data: Data,
}

/// A response with fail status.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend#fail) spec
#[derive(Debug, Serialize, Deserialize)]
pub struct Failure {
    pub status: StatusInfo,
    pub data: Data,
}

impl Feedback for Failure {
    fn feedback(&self) {
        match &self.data {
            super::response::Data::FailureMessage(reason) => {
                submission_failed(reason);
            }
            super::response::Data::UserOrganizations { organizations } => {
                // Currently nothing to do here.
            }
        }
    }
}

/// A response with error status.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend#error) spec
#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub status: StatusInfo,
    pub code: u16,
    pub error: String,
    pub message: String,
    pub data: Option<Data>,
    pub timestamp: String,
}

impl Feedback for Error {
    fn feedback(&self) {
        submission_failed(&self.message);
    }
}

/// A summary of an Auth0 organization which is returned in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrganizationSummary {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[allow(clippy::large_enum_variant)]
/// `data` field of the response.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
    FailureMessage(String),
    // Serialized as {"field": "...", }
    UserOrganizations {
        organizations: Vec<UserOrganizationSummary>,
    },
}

pub enum RegistryResponse {
    ListingSuccess {
        organizations: Vec<UserOrganizationSummary>,
    },
    Failure(Failure),
    Error(Error),
}

impl From<Error> for RegistryResponse {
    fn from(response: Error) -> Self {
        Self::Error(response)
    }
}

impl From<Failure> for RegistryResponse {
    fn from(response: Failure) -> Self {
        Self::Failure(response)
    }
}

impl Feedback for RegistryResponse {
    fn feedback(&self) {
        match self {
            Self::ListingSuccess { organizations } => {
                if organizations.is_empty() {
                    println!("\t{}", "You are member of no organizations yet.".yellow());
                    return;
                }
                for (i, organization) in organizations.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    println!(
                        "\t{} {}",
                        "Organization".green().bold(),
                        organization.display_name
                    );
                    println!("\t    {}{}", "Name: ".yellow(), organization.name);
                    println!("\t    {}{}", "Id: ".yellow(), organization.id);
                }
            }
            Self::Failure(failure) => {
                failure.feedback();
            }
            Self::Error(error) => {
                error.feedback();
            }
        }
    }
}
