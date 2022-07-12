use actix_web::{
    http::header::{
        CACHE_CONTROL, CONTENT_SECURITY_POLICY, EXPIRES, PRAGMA, X_CONTENT_TYPE_OPTIONS,
        X_FRAME_OPTIONS, X_XSS_PROTECTION,
    },
    middleware::DefaultHeaders,
};

pub fn security_headers() -> DefaultHeaders {
    let headers = DefaultHeaders::new()
        // It is recommended to have X-XSS-Protection: 0 and use the more powerful and flexible Content-Security-Policy header instead.
        // More info: https://stackoverflow.com/questions/9090577/what-is-the-http-header-x-xss-protection/57802070#57802070
        .add((X_XSS_PROTECTION, "0"))
        .add((X_FRAME_OPTIONS, "deny"))
        .add((X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .add((
            CONTENT_SECURITY_POLICY,
            "default-src 'self'; frame-ancestors 'none';",
        ))
        .add((
            CACHE_CONTROL,
            "no-cache, no-store, max-age=0, must-revalidate",
        ))
        .add((PRAGMA, "no-cache"))
        .add((EXPIRES, "0"));

    headers
}
