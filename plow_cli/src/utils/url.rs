use anyhow::{anyhow, Result};
use camino::{Utf8Path, Utf8PathBuf};
use url::Url;

/// A type that can be converted to a Url
pub trait IntoUrl {
    /// Performs the conversion
    fn into_url(self) -> Result<Url>;
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> Result<Url> {
        Url::parse(self).map_err(|s| anyhow::format_err!("invalid url `{}`: {}", self, s))
    }
}

impl<'a> IntoUrl for &'a Utf8Path {
    fn into_url(self) -> Result<Url> {
        Url::from_file_path(self).map_err(|()| anyhow::format_err!("invalid path url `{}`", self))
    }
}

impl<'a> IntoUrl for &'a Utf8PathBuf {
    fn into_url(self) -> Result<Url> {
        self.as_path().into_url()
    }
}
