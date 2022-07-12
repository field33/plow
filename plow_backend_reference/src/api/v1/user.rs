use super::response;
use crate::AppState;
use actix_http::StatusCode;
use actix_web::{get, web, HttpResponseBuilder, Responder};

use futures::lock::Mutex;

/// Gets a list of tokens which belong to a user.
///
/// ## Properties:
/// - Request type: `GET`
/// - Needs authentication: **Yes**
/// - Respects `API_SECURITY` environment variable: **No**
/// - Routes:
///   - `/v1/user/details`
/// - Request Headers:
///   - `Authorization: Bearer <access_token>` or `Authorization: Basic <base64_encoded_api_token>`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": {
///      // Fields might change in the future.
///      "tier": "<values-are-to-be-determined>",
///      "role": "<values-are-to-be-determined>",
///     },
///  }
///  ```
#[get("/details")]
pub async fn get_user_details(_data: web::Data<Mutex<AppState>>) -> impl Responder {
    let response = response::Success::new(Some(response::Data::UserDetails {
        tier: "free".to_string(),
        role: "user".to_string(),
    }));
    return HttpResponseBuilder::new(StatusCode::OK).json(response);
}
