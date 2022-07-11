//! Version 1 of the plow registry service API.

/// Endpoints concerning retrieving or creating artifacts.
pub mod artifact;
/// Endpoints concerning getting info about fields and their details.
pub mod field;
/// Module where the response types are defined and utility functions are managed for the api.
pub mod response;
/// Endpoints concerning getting info about users and their details.
pub mod user;

pub mod token;