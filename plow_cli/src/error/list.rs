use thiserror::Error;

use crate::feedback::{
    argument_not_recognized, command_not_complete, linting_failed, listing_failed,
    submission_failed, Feedback,
};

#[derive(Error, Debug)]
pub enum ListError {
    #[error(
        "Please provide a recognized argument. Check the help for the list of supported arguments."
    )]
    ArgumentNotRecognized,
    #[error("Listing request failed with status code: {code:?}")]
    RequestFailed { code: Option<reqwest::StatusCode> },
    #[error("There was no valid body in the submission response. You may check weather the registry url you have provided is right. Your current registry url is {registry_url:?}")]
    NoValidBodyInResponse { registry_url: String },
    #[error("Registry sent an invalid response which does not conform to jsend standard. Status code of the response: {code:?}")]
    InvalidResponseFromRegistry { code: reqwest::StatusCode },
    // #[error(
    //     "Please provide a valid registry path to submit to or do not use --registry command line option."
    // )]
    // RegistryPathNotProvided,
    // #[error("")]
    // LintingFailed,
    // #[error("The registry url \"{url:?}\" is invalid. Try providing a valid registry url either in Plow.toml or with a command line argument.")]
    // InvalidRegistryUrl { url: String },
    // // We can extend this to include the response body but I think it is currently not necessary.
    // #[error("Registry sent an invalid response which does not conform to jsend standard. Status code of the response: {code:?}")]
    // InvalidResponseFromRegistry { code: reqwest::StatusCode },
    // #[error("Submission request failed with status code: {code:?}")]
    // RequestFailed { code: Option<reqwest::StatusCode> },
    // #[error("There was no valid body in the submission response. You may check weather the registry url you have provided is right. Your current registry url is {registry_url:?}")]
    // NoValidBodyInResponse { registry_url: String },
}

impl Feedback for ListError {
    fn feedback(&self) {
        use ListError::*;
        match self {
            InvalidResponseFromRegistry { .. }
            | RequestFailed { .. }
            | NoValidBodyInResponse { .. } => {
                listing_failed(&format!("{self}"));
            }
            ArgumentNotRecognized => {
                argument_not_recognized(&format!("{self}"));
            }
        }
    }
}
