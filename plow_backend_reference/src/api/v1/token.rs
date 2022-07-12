pub mod types;

use super::response;
use crate::{api::v1::token::types::TokenGenerationForm, AppState};
use actix_web::{delete, get, post, web, HttpResponseBuilder, Responder};

use crate::api::v1::response::types::ApiTokenSummary;
use futures::lock::Mutex;
use reqwest::StatusCode;

const FIXED_TOKEN: &str = "token123";

fn fixed_api_token_summary() -> ApiTokenSummary {
    ApiTokenSummary {
        id: 0,
        name: "Fixed token".to_string(),
        expires: -1,
        created_at: "2020-01-01T00:00:00.000Z".to_string(),
        last_used_at: None,
    }
}

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
    _requested_token: web::Form<TokenGenerationForm>,
    _data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    HttpResponseBuilder::new(StatusCode::OK).json(response::Success::new(Some(
        response::Data::GeneratedApiToken {
            tokens: vec![fixed_api_token_summary()],
            generated_token: base64::encode(FIXED_TOKEN),
        },
    )))
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
pub async fn get_token_summaries(_data: web::Data<Mutex<AppState>>) -> impl Responder {
    let response = response::Success::new(Some(response::Data::ApiTokens(vec![
        fixed_api_token_summary(),
    ])));
    HttpResponseBuilder::new(StatusCode::OK).json(response)
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
    _path: web::Path<(String,)>,
    _data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    HttpResponseBuilder::new(StatusCode::OK).json(response::Success::new(Some(
        response::Data::ApiTokens(vec![]),
    )))
}
