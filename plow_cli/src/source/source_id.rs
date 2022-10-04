use crate::utils::url::IntoUrl;
use camino::Utf8Path;
use serde::de;
use serde::ser;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{self, Formatter};
use std::hash::{self, Hash};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Mutex;
use url::Url;

use anyhow::anyhow;
use anyhow::Result;

lazy_static::lazy_static! {
    static ref SOURCE_ID_CACHE: Mutex<HashSet<&'static SourceIdInner>> = Default::default();
}

/// Unique identifier for a source of packages.
#[derive(Clone, Copy, Eq, Debug)]
pub struct SourceId {
    inner: &'static SourceIdInner,
}

#[derive(Eq, Clone, Debug)]
struct SourceIdInner {
    url: Url,
    kind: SourceKind,
    name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SourceKind {
    Registry,
}

impl SourceId {
    fn new(kind: SourceKind, url: Url, name: Option<&str>) -> Result<SourceId> {
        let source_id = SourceId::wrap(SourceIdInner {
            kind,
            url,
            name: name.map(|name| name.into()),
        });
        Ok(source_id)
    }

    fn wrap(inner: SourceIdInner) -> SourceId {
        let mut cache = SOURCE_ID_CACHE.lock().unwrap();
        let inner = cache.get(&inner).cloned().unwrap_or_else(|| {
            let inner = Box::leak(Box::new(inner));
            cache.insert(inner);
            inner
        });
        SourceId { inner }
    }

    /// Parses a source URL and returns the corresponding ID.
    ///
    /// ## Example
    ///
    /// ```
    /// use cargo::core::SourceId;
    /// SourceId::from_url("git+https://github.com/alexcrichton/\
    ///                     libssh2-static-sys#80e71a3021618eb05\
    ///                     656c58fb7c5ef5f12bc747f");
    /// ```
    pub fn from_url(input: &str) -> Result<SourceId> {
        let (mut kind, mut url) = input.split_once('+').ok_or_else(|| {
            anyhow!(
                "invalid source ID `{input}`: expected \
                 `kind+url`",
            )
        })?;

        match kind {
            "registry" => {
                let url = url.into_url()?;
                Ok(SourceId::new(SourceKind::Registry, url, None)?)
            }
            kind => Err(anyhow::format_err!("unsupported source protocol: {}", kind)),
        }
    }

    /// A view of the `SourceId` that can be `Display`ed as a URL.
    pub fn as_url(&self) -> SourceIdAsUrl<'_> {
        SourceIdAsUrl {
            inner: &*self.inner,
        }
    }

    /// Creates a SourceId from a remote registry URL when the registry name
    /// cannot be determined, e.g. a user passes `--index` directly from CLI.
    ///
    /// Use [`SourceId::for_alt_registry`] if a name can provided, which
    /// generates better messages for cargo.
    pub fn for_registry(url: &Url) -> Result<SourceId> {
        SourceId::new(SourceKind::Registry, url.clone(), None)
    }

    // TODO: Needs re architecting to separate lib and bin parts
    /// Returns the `SourceId` corresponding to the main repository.
    ///
    /// This is the main plow registry by default, but it can be overridden in
    /// a `.plow/config.toml`.
    // pub fn plow_public_registry_index(config: &PlowConfig) -> Result<SourceId> {
    //     todo!()
    // }

    // TODO: Needs re architecting to separate lib and bin parts
    /// Returns the `SourceId` corresponding to the main repository.
    ///
    /// This is the main plow registry by default, but it can be overridden in
    /// a `.plow/config.toml`.
    // pub fn plow_private_registry_index(config: &PlowConfig) -> Result<SourceId> {
    //     todo!()
    // }

    // TODO: Check implementation
    /// Gets the `SourceId` associated with given name of the remote registry.
    // pub fn alt_registry(config: &Config, key: &str) -> CargoResult<SourceId> {
    //     let url = config.get_registry_index(key)?;
    //     Ok(SourceId::wrap(SourceIdInner {
    //         kind: SourceKind::Registry,
    //         url,
    //         name: Some(key.to_string()),
    //     }))
    // }

    /// Gets this source URL.
    pub fn url(&self) -> &Url {
        &self.inner.url
    }

    // TODO: Check
    // pub fn display_index(self) -> String {
    //     if self.is_default_registry() {
    //         format!("{} index", CRATES_IO_DOMAIN)
    //     } else {
    //         format!("`{}` index", self.display_registry_name())
    //     }
    // }

    // TODO: Check
    // pub fn display_registry_name(self) -> String {
    //     if self.is_default_registry() {
    //         CRATES_IO_REGISTRY.to_string()
    //     } else if let Some(name) = &self.inner.name {
    //         name.clone()
    //     } else if self.precise().is_some() {
    //         // We remove `precise` here to retrieve an permissive version of
    //         // `SourceIdInner`, which may contain the registry name.
    //         self.with_precise(None).display_registry_name()
    //     } else {
    //         url_display(self.url())
    //     }
    // }

    /// Returns `true` if this source is from a registry (either local or not).
    pub fn is_registry(self) -> bool {
        matches!(self.inner.kind, SourceKind::Registry)
    }

    /// Returns `true` if this source is a "remote" registry.
    ///
    /// "remote" may also mean a file URL to a git index, so it is not
    /// necessarily "remote". This just means it is not `local-registry`.
    pub fn is_remote_registry(self) -> bool {
        matches!(self.inner.kind, SourceKind::Registry)
    }

    // TODO: Implement and check
    /// Creates an implementation of `Source` corresponding to this ID.
    pub fn load<'a>(
        self,
        // config: &'a Config,
        // yanked_whitelist: &HashSet<PackageId>,
    ) -> Result<Box<dyn super::Source + 'a>> {
        match self.inner.kind {
            SourceKind::Registry => {
                todo!()
            }
        }
    }

    /// Returns `true` if the remote registry is the standard <https://plow.pm>.
    pub fn is_default_registry(self) -> bool {
        todo!()
    }

    pub fn stable_hash<S: hash::Hasher>(self, workspace: &Path, into: &mut S) {
        self.hash(into)
    }

    pub fn full_eq(self, other: SourceId) -> bool {
        ptr::eq(self.inner, other.inner)
    }

    pub fn full_hash<S: hash::Hasher>(self, into: &mut S) {
        ptr::NonNull::from(self.inner).hash(into)
    }
}

impl PartialEq for SourceId {
    fn eq(&self, other: &SourceId) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for SourceId {
    fn partial_cmp(&self, other: &SourceId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceId {
    fn cmp(&self, other: &SourceId) -> Ordering {
        // If our interior pointers are to the exact same `SourceIdInner` then
        // we're guaranteed to be equal.
        if ptr::eq(self.inner, other.inner) {
            return Ordering::Equal;
        }

        // Sort first based on `kind`, deferring to the URL comparison below if
        // the kinds are equal.
        match self.inner.kind.cmp(&other.inner.kind) {
            Ordering::Equal => {}
            other => return other,
        }
    }
}

impl ser::Serialize for SourceId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.collect_str(&self.as_url())
    }
}

impl<'de> de::Deserialize<'de> for SourceId {
    fn deserialize<D>(d: D) -> Result<SourceId, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let string = String::deserialize(d)?;
        SourceId::from_url(&string).map_err(de::Error::custom)
    }
}

fn url_display(url: &Url) -> String {
    url.as_str().to_string()
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.inner.kind {
            SourceKind::Registry => write!(f, "registry `{}`", self.display_registry_name()),
        }
    }
}

impl Hash for SourceId {
    fn hash<S: hash::Hasher>(&self, into: &mut S) {
        self.inner.kind.hash(into);
        match self.inner.kind {
            _ => self.inner.url.as_str().hash(into),
        }
    }
}

impl Hash for SourceIdInner {
    /// The hash of `SourceIdInner` is used to retrieve its interned value. We
    /// only care about fields that make `SourceIdInner` unique, which are:
    ///
    /// - `kind`
    /// - `url` was canonical url in orig cargo source
    fn hash<S: hash::Hasher>(&self, into: &mut S) {
        self.kind.hash(into);
        self.url.hash(into);
    }
}

impl PartialEq for SourceIdInner {
    /// This implementation must be synced with [`SourceIdInner::hash`].
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.url == other.url
    }
}

// forward to `Ord`
impl PartialOrd for SourceKind {
    fn partial_cmp(&self, other: &SourceKind) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceKind {
    fn cmp(&self, other: &SourceKind) -> Ordering {
        match (self, other) {
            (SourceKind::Registry, SourceKind::Registry) => Ordering::Equal,
            (SourceKind::Registry, _) => Ordering::Less,
            (_, SourceKind::Registry) => Ordering::Greater,
        }
    }
}

/// A `Display`able view into a `SourceId` that will write it as a url
pub struct SourceIdAsUrl<'a> {
    inner: &'a SourceIdInner,
}

impl<'a> fmt::Display for SourceIdAsUrl<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self.inner {
            SourceIdInner {
                kind: SourceKind::Registry,
                ref url,
                ..
            } => write!(f, "registry+{}", url),
        }
    }
}
