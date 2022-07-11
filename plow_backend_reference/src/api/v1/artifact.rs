#![allow(clippy::unused_async)]

use super::response;
use crate::{
    AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
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
    path: web::Path<(String,)>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}
