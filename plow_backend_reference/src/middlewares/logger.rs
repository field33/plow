use actix_web::middleware::Logger;

#[allow(dead_code)]
/// Log levels
pub enum Level {
    Debug,
    Info,
    Warn,
}

#[allow(clippy::from_over_into)]
impl Into<String> for Level {
    fn into(self) -> String {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
        }
        .to_owned()
    }
}

/// Either reads the log level from the environment variable `ACTIX_LOG_LEVEL`
/// or falls back to the default log level which is provided in this function.
pub fn read_level_or_default(default: Level) {
    std::env::set_var(
        "RUST_LOG",
        &std::env::var("ACTIX_LOG_LEVEL").unwrap_or_else(|_| default.into()),
    );
}

#[allow(dead_code)]
/// Overrides and sets log level explicitly.
pub fn set_level(default: Level) {
    std::env::set_var(
        "RUST_LOG",
        &std::env::var("ACTIX_LOG_LEVEL").unwrap_or_else(|_| default.into()),
    );
}

pub fn logger() -> Logger {
    Logger::default()
}
