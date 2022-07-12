#![allow(clippy::unused_async)]

use super::response;
use crate::AppState;
use actix_http::StatusCode;
use actix_web::{get, web, HttpResponseBuilder, Responder};
use futures::lock::Mutex;

/// Gets a signed url for intended artifact for public download functionality.
///
/// ## Properties:
/// - Request type: `GET`
/// - Needs authentication: **Yes**
/// - Respects `API_SECURITY` environment variable: **Yes**
/// - Routes:
///   - `/v1/artifact/signed-url/{artifact_name}`
/// - Request Headers:
///   - `Authorization: Bearer <access_token>` or `Authorization: Basic <base64_encoded_api_token>`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": {
///      "url": "<a-signed-download-url-for-the-artifact>",
///    },
///  }
///  ```
#[get("/signed-url/{artifact_name}")]
pub async fn get_signed_url(
    _path: web::Path<(String,)>,
    _data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let response = response::Success::new(Some(response::Data::ResourceUrl {
        url: "http://example.com",
    }));
    HttpResponseBuilder::new(StatusCode::OK).json(response)
}
