#![allow(clippy::items_after_statements)]
mod types;

use super::response;
use crate::{
    api::v1::field::{
        types::FieldSearchForm,
    },
    AppState,
};
use actix_web::{get, post, web, Responder};

use futures::lock::Mutex;

/// Gets all field categories in an alphabetically sorted way.
///
/// ## Properties:
/// - Request type: `GET`
/// - Needs authentication: **No**
/// - Routes:
///   - `/v1/field/categories`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": [
///      {
///        "name": "<category name>",
///        "description": "<category_description>",
///      },
///      //...
///    ],
///  }
///  ```
#[get("/categories")]
pub async fn get_all_categories(data: web::Data<Mutex<AppState>>) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Deliver all fields which belongs to the user, descending sort.
///
/// ## Properties:
/// - Request type: `GET`
/// - Needs authentication: **Yes**
/// - Routes:
///   - `/v1/field/private/list/{auth_id}`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": [
///      {
///        "id": "<field hash>",
///        "namespace": "<field namespace>",
///        "name": "<field name>",
///        "version": "<field version>",
///        "categories": [
///          {
///            "name": "<category name>",
///            "description": "<category description>",
///          },
///          //...
///        ],
///        "submitted_at": "<timestamp>",
///        "submitter_id": "<field submitter id (corresponds to user id)>",
///        "submitter_name": "<field submitter name (readable name of the submitter)>",
///      },
///      //...
///    ],
///  }
///  ```
#[get("/list/{auth_id}")]
pub async fn get_fields_which_belong_to_a_user(
    path: web::Path<(String,)>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Get details of a field by its id.
///
/// ## Properties:
/// - Request type: `GET`
/// - Needs authentication: **No**
/// - Routes:
///   - `/v1/field/private/details/{field_id}`
/// ## Example Successful Response:
///  ```json
///  {
///    "status": "success",
///    "data": {
///      title: "<readable title of a field>",
///      short_description: "<readable short description of a field>",
///      description: "<readable description of a field>",
///      state: "<state of a field (values will be included when they are stabilized)>",
///      license_spdx: "<license of a field>",
///      homepage: "<homepage of a field>",
///      documentation: "<documentation of a field>",
///      repository: "<repository of a field>",
///      keywords: [
///        "<keyword>",
///        //...
///      ],
///      versions: [
///        {
///          "id": "<field hash>",
///          "namespace": "<field namespace>",
///          "name": "<field name>",
///          "version": "<field version>",
///          "categories": [
///            {
///              "name": "<category name>",
///              "description": "<category description>",
///            },
///            //...
///          ],
///          "submitted_at": "<timestamp>",
///          "submitter_id": "<field submitter id (corresponds to user id)>",
///          "submitter_name": "<field submitter name (readable name of the submitter)>",
///        },
///        //...
///      ],
///      dependencies: [
///        {
///          "id": "<field hash>",
///          "namespace": "<field namespace>",
///          "name": "<field name>",
///          "version": "<field version>",
///          "categories": [
///            {
///              "name": "<category name>",
///              "description": "<category description>",
///            },
///            //...
///          ],
///          "submitted_at": "<timestamp>",
///          "submitter_id": "<field submitter id (corresponds to user id)>",
///          "submitter_name": "<field submitter name (readable name of the submitter)>",
///        },
///        //...
///      ],
///      // Dependencies and dependents may be empty if there are no dependencies or dependents.
///      dependents: []
///    }
///  }
///  ```
#[get("/details/{field_id}")]
pub async fn get_field_details_with_field_id(
    path: web::Path<(String,)>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Lists new and recent fields optionally specifying a range of dates.
///
/// ## Routes
/// - `/v1/field/list/time_scoped?new_fields_from=<RFC3339 compliant timestamp string>`
/// - `/v1/field/list/time_scoped?recent_fields_from=<RFC3339 compliant timestamp string>`
/// - `/v1/field/list/time_scoped?new_fields_from=<RFC3339 compliant timestamp string>&recent_fields_from=<RFC3339 compliant timestamp string>`
/// - `/v1/field/list/time_scoped`
///
/// If timestamps are not provided, the default is to return the last 24 hours for both.
#[get("/list/time_scoped")]
pub async fn list_new_and_recent_time_scoped(
    query_parameters: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Lists new and recent fields.
///
/// A field is considered new when it only has a single version published.
///
/// A field is considered recent when there are previous versions of the field already.
///
/// This endpoint returns the last N fields in these categories.
/// The sort is happening by the last submission date.
///
/// ## Routes
///   - `/v1/field/list?last_n_fields=<number>`
///   - `/v1/field/list`
///
/// If a number is not provided then the last 5 fields will be returned.
#[allow(clippy::too_many_lines)]
#[get("/list")]
pub async fn list_new_and_recent_n_fields(
    query_parameters: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}

/// Searches fields by their name.
///
/// ## Routes
///   - `/v1/field/search?name=<anything>`
#[post("/search")]
pub async fn search(
    search: web::Form<FieldSearchForm>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    unimplemented!();
    response::with_error_message("Unimplemented")
}
