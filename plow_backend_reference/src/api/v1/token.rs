pub mod types;

use super::response;
use crate::{
    api::v1::token::{
        types::TokenGenerationForm,
    },
    AppState,
};
use actix_web::{delete, get, post, web, HttpResponse, HttpResponseBuilder, Responder};

use futures::lock::Mutex;
use reqwest::StatusCode;

/// Generates an api token for an app client.
///
/// ## Properties:
/// - Request type: `POST`
/// - Needs authentication: **Yes**
/// - Respects `API_SECURITY` environment variable: **No**
/// - Routes:
///   - `/v1/token/generate`
/// - Request Headers:
///   - `Content-Type: application/x-www-form-urlencoded`
///   - `Authorization: Bearer <access_token>` or `Authorization: Basic <base64_encoded_api_token>`
/// - Request Parameters:
///   - `name`: The name of the token.
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": {
///      // List of tokens which belong to the querying user.
///      "tokens": [
///         {
///           "id": 1,
///           "name": "My Token",
///           "expires": -1, // Infinite
///           "created_at": "2020-01-01T00:00:00.000Z",
///           "last_used_at": null, // Nullable
///         },
///         // ...
///      ],
///      "generated_token": "<base64-encoded-string>"
///    }
///  }
///  ```
#[post("/generate")]
pub async fn generate_api_token(
    requested_token: web::Form<TokenGenerationForm>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Gets a list of tokens which belong to a user.
///
/// ## Properties:
/// - Request type: `GET`
/// - Needs authentication: **Yes**
/// - Respects `API_SECURITY` environment variable: **No**
/// - Routes:
///   - `/v1/token/list`
/// - Request Headers:
///   - `Authorization: Bearer <access_token>` or `Authorization: Basic <base64_encoded_api_token>`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": [
///      {
///        "id": 1,
///        "name": "My Token",
///        "expires": -1, // Infinite
///        "created_at": "2020-01-01T00:00:00.000Z",
///        "last_used_at": null, // Nullable
///      },
///      // ...
///    ],
///  }
///  ```
#[get("/list")]
pub async fn get_token_summaries(
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Deletes a token by it's id.
///
/// ## Properties:
/// - Request type: `DELETE`
/// - Needs authentication: **Yes**
/// - Respects `API_SECURITY` environment variable: **No**
/// - Routes:
///   - `/v1/token/{token_id}`
/// - Request Headers:
///   - `Authorization: Bearer <access_token>` or `Authorization: Basic <base64_encoded_api_token>`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": null
///  }
///  ```
#[delete("/{token_id}")]
pub async fn delete_api_token_by_its_id(
    path: web::Path<(String,)>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}
