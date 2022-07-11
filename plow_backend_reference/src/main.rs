#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,
    clippy::non_ascii_literal,
    clippy::print_stdout,
    clippy::print_stderr,

    // We decided that we're ok with expect
    clippy::expect_used,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,
    // Currently breaks CI, let's wait a bit more until new clippy version is more spread.
    // clippy::single_char_lifetime_names,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    clippy::wildcard_dependencies,
    clippy::redundant_feature_names,
    clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,

    clippy::negative_feature_names
)]
#![allow(dead_code)]

use actix_web::{web, App, HttpServer};

use anyhow::Result;

use dotenv::dotenv;
use futures::lock::Mutex;
use service::env::{
    ACTIX_PORT
};
use service::middlewares::logger::Level;
use service::{api, middlewares, AppState};

fn server_url() -> String {
    let mut host = "0.0.0.0:".to_owned();
    host.push_str(&ACTIX_PORT);
    host
}

#[actix_web::main]
async fn main() -> Result<()> {
    // env
    dotenv().ok();

    // logs
    middlewares::logger::read_level_or_default(Level::Info);
    env_logger::init();

    // app data
    let data = web::Data::new(Mutex::new(AppState { }));

    // server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&data))
            .wrap(middlewares::logger())
            .wrap(middlewares::cors::allow_list())
            .wrap(middlewares::security_headers())
            // Health check endpoints
            .route("/", web::get().to(actix_web::HttpResponse::Ok))
            .route("/health-check", web::get().to(actix_web::HttpResponse::Ok))
            .service(
                web::scope(api::Version::V1.as_path())
                    .service(web::scope("/artifact").service(api::v1::artifact::get_signed_url))
                    .service(
                        web::scope("/field")
                            .service(api::v1::field::search)
                            .service(api::v1::field::get_all_categories)
                            .service(api::v1::field::list_new_and_recent_time_scoped)
                            .service(api::v1::field::list_new_and_recent_n_fields)
                            .service(api::v1::field::get_field_details_with_field_id)
                            .service(api::v1::field::get_fields_which_belong_to_a_user),
                    )
                    .service(web::scope("/user").service(api::v1::user::get_user_details))
                    .service(
                        web::scope("/token")
                            .service(api::v1::token::generate_api_token)
                            .service(api::v1::token::get_token_summaries)
                            .service(api::v1::token::delete_api_token_by_its_id),
                    ),
            )

    });

    server.bind(server_url())?.run().await?;

    Ok(())
}
