use std::{collections::HashSet, hash, sync::Mutex};

use crate::registry::SemanticVersion;
use lazy_static::lazy_static;
use ustr::Ustr;

lazy_static! {
    static ref FIELD_ID_CACHE: Mutex<HashSet<&'static FieldIdInner>> = Mutex::new(HashSet::new());
}

/// Identifier for a specific version of a Field in a specific source.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct FieldId {
    inner: &'static FieldIdInner,
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
struct FieldIdInner {
    name: Ustr,
    version: SemanticVersion,
    // source_id: SourceId,
}

impl PartialEq for FieldId {
    fn eq(&self, other: &FieldId) -> bool {
        if std::ptr::eq(self.inner, other.inner) {
            return true;
        }
        // This is here so that PackageId uses SourceId's and Version's idea
        // of equality. PackageIdInner uses a more exact notion of equality.
        self.inner.name == other.inner.name && self.inner.version == other.inner.version
        // && self.inner.source_id == other.inner.source_id
    }
}

impl std::hash::Hash for FieldId {
    fn hash<S: hash::Hasher>(&self, state: &mut S) {
        // This is here (instead of derived) so that PackageId uses SourceId's
        // and Version's idea of equality. PackageIdInner uses a more exact
        // notion of hashing.
        self.inner.name.hash(state);
        self.inner.version.hash(state);
        // self.inner.source_id.hash(state);
    }
}

impl FieldId {
    pub fn new(name: &str, version: SemanticVersion /*sid: SourceId*/) -> FieldId {
        let inner = FieldIdInner {
            name: Ustr::from(name),
            version,
            /*  source_id,*/
        };
        let mut cache = FIELD_ID_CACHE.lock().unwrap();
        let inner = cache.get(&inner).cloned().unwrap_or_else(|| {
            let inner = Box::leak(Box::new(inner));
            cache.insert(inner);
            inner
        });
        FieldId { inner }
    }

    pub fn name(self) -> Ustr {
        self.inner.name
    }
    pub fn version(self) -> &'static SemanticVersion {
        &self.inner.version
    }

    // pub fn source_id(self) -> SourceId {
    //     self.inner.source_id
    // }
}

impl core::fmt::Display for FieldId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} v{}", self.inner.name, self.inner.version)?;

        // if !self.inner.source_id.is_default_registry() {
        //     write!(f, " ({})", self.inner.source_id)?;
        // }

        Ok(())
    }
}
