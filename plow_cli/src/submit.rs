pub mod response {
    use anyhow::bail;
    use serde::{Deserialize, Serialize};

    /// `status` field of the response.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub enum StatusInfo {
        #[serde(rename(serialize = "success", deserialize = "success"))]
        Success,
        #[serde(rename(serialize = "fail", deserialize = "fail"))]
        Failure,
        #[serde(rename(serialize = "error", deserialize = "error"))]
        Error,
    }

    impl TryFrom<&str> for StatusInfo {
        type Error = anyhow::Error;
        fn try_from(s: &str) -> Result<Self, anyhow::Error> {
            match s {
                "success" => Ok(StatusInfo::Success),
                "fail" => Ok(StatusInfo::Failure),
                "error" => Ok(StatusInfo::Error),
                s => bail!("Invalid status text: {}", s),
            }
        }
    }
    /// A response with success status.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend#success) spec
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Success<'body> {
        pub status: StatusInfo,
        #[serde(borrow)]
        pub data: Option<Data<'body>>,
    }

    /// A response with fail status.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend#fail) spec
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Failure<'body> {
        pub status: StatusInfo,
        #[serde(borrow)]
        pub data: Data<'body>,
    }

    /// A response with error status.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend#error) spec
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Error<'body> {
        pub status: StatusInfo,
        pub code: u16,
        pub error: &'body str,
        pub message: &'body str,
        pub data: Option<Data<'body>>,
        pub timestamp: String,
    }

    #[allow(clippy::large_enum_variant)]
    /// `data` field of the response.
    ///
    /// Following [`JSend`](https://github.com/omniti-labs/jsend) spec
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Data<'data> {
        FailureMessage(&'data str),
        // Serialized as {"field": "...", }
        SubmissionLintingResults {
            /// Non exhaustive list of linting failure messages.
            failures: Vec<String>,
        },
    }
}
