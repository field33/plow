use actix_cors::Cors;

pub fn allow_list() -> Cors {
    Cors::default()
        .supports_credentials()
        .allowed_origin("http://localhost")
        .allowed_origin("http://localhost:3000")
        .allowed_origin("http://localhost:8080")
        .allow_any_method()
        .allow_any_header()
        .max_age(86_400)
}
