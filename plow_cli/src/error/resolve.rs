use crate::feedback::{dependency_resolution_failed, Feedback};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ResolveError {
    #[error("{0}")]
    FailedToResolveDependencies(String),
}

impl Feedback for ResolveError {
    fn feedback(&self) {
        use ResolveError::*;
        match self {
            FailedToResolveDependencies(_) => {
                dependency_resolution_failed(&format!("{self}"));
            }
        }
    }
}
