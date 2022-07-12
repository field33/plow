//! Data structures to generate JSON responses.

#![allow(clippy::unused_async)]
#![allow(clippy::use_self)]

pub mod types;

use actix_web::HttpResponse;
use actix_web::HttpResponseBuilder;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;

use types::Category;
use types::FieldSummary;

use self::types::ApiTokenSummary;

/// A response with success status.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend#success) spec
#[derive(Debug, Serialize, Deserialize)]
pub struct Success<'body> {
    pub status: StatusInfo,
    #[serde(borrow)]
    pub data: Option<Data<'body>>,
}

impl<'body> Success<'body> {
    pub const fn new(data: Option<Data<'body>>) -> Self {
        Self {
            status: StatusInfo::Success,
            data,
        }
    }
}

/// A response with fail status.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend#fail) spec
#[derive(Debug, Serialize, Deserialize)]
pub struct Failure<'body> {
    pub status: StatusInfo,
    #[serde(borrow)]
    pub data: Data<'body>,
}
impl<'body> Failure<'body> {
    pub const fn new(data: Data<'body>) -> Self {
        Self {
            status: StatusInfo::Failure,
            data,
        }
    }
}

/// A response with error status.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend#error) spec
#[derive(Debug, Serialize, Deserialize)]
pub struct Error<'body> {
    pub status: StatusInfo,
    pub code: u16,
    pub error: &'body str,
    pub message: &'body str,
    pub data: Option<Data<'body>>,
    pub timestamp: String,
}

impl<'body> Error<'body> {
    pub fn new(status: StatusCode, message: &'body str, data: Option<Data<'body>>) -> Self {
        Self {
            status: StatusInfo::Error,
            code: status.as_u16(),
            error: status.canonical_reason().unwrap_or(""),
            message,
            data,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// `status` field of the response.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum StatusInfo {
    #[serde(rename(serialize = "success", deserialize = "success"))]
    Success,
    #[serde(rename(serialize = "fail", deserialize = "fail"))]
    Failure,
    #[serde(rename(serialize = "error", deserialize = "error"))]
    Error,
}

#[allow(clippy::large_enum_variant)]
/// `data` field of the response.
///
/// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Data<'data> {
    FailureMessage(&'data str),
    Categories(Vec<Category>),
    FieldSummaries(Vec<FieldSummary>),
    // Serialized as {"field": "...", }
    NewAndRecentFields {
        new_fields: Vec<FieldSummary>,
        recent_fields: Vec<FieldSummary>,
    },
    UserDetails {
        tier: String,
        role: String,
    },
    ApiTokens(Vec<ApiTokenSummary>),
    GeneratedApiToken {
        /// Updated list of tokens
        tokens: Vec<ApiTokenSummary>,
        /// Base 64 encoded un-encrypted publish token.
        generated_token: String,
    },
    FieldDetails {
        title: &'data str,
        short_description: &'data str,
        description: &'data str,
        // state: entity::field::State,
        license_spdx: &'data str,
        homepage: &'data str,
        documentation: &'data str,
        repository: &'data str,
        keywords: Vec<&'data str>,
        versions: Vec<FieldSummary>,
        dependencies: Vec<FieldSummary>,
        dependents: Vec<FieldSummary>,
    },
    ResourceUrl {
        url: &'data str,
    },
    ResourceName {
        name: &'data str,
    },
    ResourceAuthHeader {
        header: &'data str,
    },
    ResourceAccessToken {
        token: &'data str,
    },
}

/// An internal server error with a message and no data attached.
pub fn with_error_message<T: AsRef<str>>(message: T) -> HttpResponse {
    let response = Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        // Details are not necessary for the client to understand.
        message.as_ref(),
        None,
    );
    HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(response)
}

/// Returns a failure response with the given message and status code.
pub fn with_failure_message<T: AsRef<str>>(message: T, code: StatusCode) -> HttpResponse {
    let response = Failure::new(Data::FailureMessage(message.as_ref()));
    HttpResponseBuilder::new(code).json(response)
}

/// This endpoint does not respect to `API_SECURITY` environment variable.
pub fn overrides_api_security_flag() -> HttpResponse {
    let response = crate::api::v1::response::Error::new(
        StatusCode::UNAUTHORIZED,
        "The endpoint which you're requesting from can not be used with disabled api security.",
        None,
    );
    HttpResponseBuilder::new(StatusCode::UNAUTHORIZED).json(response)
}

/// A success response with no data attached.
pub fn success_with_null_data() -> HttpResponse {
    let response = Success::new(None);
    HttpResponseBuilder::new(StatusCode::OK).json(response)
}

/// A function to convert status codes to generic messages.
pub fn generic_message_from_status_code(status_code: StatusCode) -> &'static str {
    match status_code.as_u16() {
        100..=199 => "Information",
        200..=299 => "Success",
        300..=399 => "Redirection",
        400..=499 => "Client Error",
        500..=599 => "Server Error",
        _ => unreachable!(),
    }
}
