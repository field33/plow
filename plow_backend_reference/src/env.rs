use lazy_static::lazy_static;
use std::env;

/// Default port for the server.
pub const DEFAULT_ACTIX_PORT: &str = "80";

lazy_static! {
    /// Port for the server.
    pub static ref ACTIX_PORT: String =
        env::var("ACTIX_PORT").unwrap_or_else(|_| DEFAULT_ACTIX_PORT.into());
}
