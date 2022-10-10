use std::rc::Rc;

use camino::{Utf8Path, Utf8PathBuf};
use ustr::Ustr;

use crate::{
    field_id::FieldId,
    manifest::{FieldAuthor, FieldManifest, FieldSummary},
    registry::{Dependency, SemanticVersion},
};

/// Information about a field that is available somewhere in the file system.
#[derive(Clone)]
pub struct Field<'fld> {
    inner: Rc<FieldInner<'fld>>,
}

#[derive(Clone)]
struct FieldInner<'fld> {
    manifest: FieldManifest<'fld>,
    path: Utf8PathBuf,
}

impl<'fld> Field<'_> {
    /// Creates a field from the manifest.
    pub fn new(manifest: FieldManifest<'fld>, field_path: &Utf8Path) -> Field<'fld> {
        Field {
            inner: Rc::new(FieldInner {
                manifest,
                path: field_path.to_path_buf(),
            }),
        }
    }

    pub fn dependencies(&self) -> &[Dependency<SemanticVersion>] {
        self.manifest().dependencies()
    }

    pub fn full_name(&self) -> Ustr {
        self.field_id().name()
    }

    pub fn manifest(&self) -> &FieldManifest {
        &self.inner.manifest
    }

    pub fn path(&self) -> &Utf8Path {
        &self.inner.path
    }

    // Gets the `FieldId` object for the field.
    //
    // This is enough to identify a field.
    pub fn field_id(&self) -> FieldId {
        self.manifest().field_id()
    }

    /// Gets the root folder of the field.
    pub fn root(&self) -> &Utf8Path {
        self.path().parent().unwrap()
    }

    /// Gets the summary for the field.
    pub fn summary(&self) -> &FieldSummary {
        self.manifest().summary()
    }

    /// Gets the current field version.
    pub fn version(&self) -> &SemanticVersion {
        self.field_id().version()
    }

    /// Gets the field authors.
    pub fn authors(&self) -> &[FieldAuthor] {
        &self.manifest().authors()
    }
}

impl core::fmt::Display for Field<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.summary().field_id())
    }
}

impl core::fmt::Debug for Field<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Field")
            .field("id", &self.summary().field_id())
            .field("..", &"..")
            .finish()
    }
}

impl<'fld> PartialEq for Field<'_> {
    fn eq(&self, other: &Field) -> bool {
        self.field_id() == other.field_id()
    }
}

impl<'fld> Eq for Field<'_> {}

impl<'fld> Ord for Field<'_> {
    fn cmp(&self, other: &Field) -> core::cmp::Ordering {
        self.field_id().cmp(&other.field_id())
    }
}

impl<'fld> PartialOrd for Field<'_> {
    fn partial_cmp(&self, other: &Field) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::hash::Hash for Field<'_> {
    fn hash<H: core::hash::Hasher>(&self, into: &mut H) {
        self.field_id().hash(into)
    }
}
