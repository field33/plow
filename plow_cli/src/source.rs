mod source_id;

use std::collections::hash_map::HashMap;
use std::fmt;

use crate::{
    error::CliError,
    field::Field,
    field_id::FieldId,
    manifest::FieldSummary,
    registry::{Dependency, SemanticVersion},
};

pub use self::source_id::SourceId;

/// Something that finds and downloads remote packages based on names and versions.
pub trait Source {
    /// Returns the `SourceId` corresponding to this source.
    fn source_id(&self) -> SourceId;

    /// Returns the replaced `SourceId` corresponding to this source.
    fn replaced_source_id(&self) -> SourceId {
        self.source_id()
    }

    /// Attempts to find a range of packages that match a dependency request.
    fn query(&mut self, dep: &Dependency<SemanticVersion>) -> Vec<FieldSummary>;

    // TODO: The error type will be updated later
    /// Fetches the full package for each name and version specified.
    fn retrieve(&mut self, id: FieldId) -> Result<MaybeField, CliError>;

    /// Describes this source in a human readable fashion, used for display in
    /// resolver error messages currently.
    fn describe(&self) -> String;

    /// Returns whether a source is being replaced by another here.
    fn is_replaced(&self) -> bool {
        false
    }
}

pub enum MaybeField {
    Ready(Field<'static>),
    Download { url: String },
}

impl<'a, T: Source + ?Sized + 'a> Source for Box<T> {
    /// Forwards to `Source::source_id`.
    fn source_id(&self) -> SourceId {
        (**self).source_id()
    }

    /// Forwards to `Source::replaced_source_id`.
    fn replaced_source_id(&self) -> SourceId {
        (**self).replaced_source_id()
    }

    /// Forwards to `Source::query`.
    fn query(&mut self, dep: &Dependency<SemanticVersion>) -> Vec<FieldSummary> {
        (**self).query(dep)
    }

    /// Forwards to `Source::download`.
    fn retrieve(&mut self, id: FieldId) -> Result<MaybeField, CliError> {
        (**self).retrieve(id)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }

    fn is_replaced(&self) -> bool {
        (**self).is_replaced()
    }
}

impl<'a, T: Source + ?Sized + 'a> Source for &'a mut T {
    /// Forwards to `Source::source_id`.
    fn source_id(&self) -> SourceId {
        (**self).source_id()
    }

    /// Forwards to `Source::replaced_source_id`.
    fn replaced_source_id(&self) -> SourceId {
        (**self).replaced_source_id()
    }

    /// Forwards to `Source::query`.
    fn query(&mut self, dep: &Dependency<SemanticVersion>) -> Vec<FieldSummary> {
        (**self).query(dep)
    }

    /// Forwards to `Source::download`.
    fn retrieve(&mut self, id: FieldId) -> Result<MaybeField, CliError> {
        (**self).retrieve(id)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }

    fn is_replaced(&self) -> bool {
        (**self).is_replaced()
    }
}

/// A `HashMap` of `SourceId` -> `Box<Source>`.
#[derive(Default)]
pub struct SourceMap<'src> {
    map: HashMap<SourceId, Box<dyn Source + 'src>>,
}

// `impl Debug` on source requires specialization, if even desirable at all.
impl<'src> fmt::Debug for SourceMap<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SourceMap ")?;
        f.debug_set().entries(self.map.keys()).finish()
    }
}

impl<'src> SourceMap<'src> {
    /// Creates an empty map.
    pub fn new() -> SourceMap<'src> {
        SourceMap {
            map: HashMap::new(),
        }
    }

    /// Like `HashMap::get`.
    pub fn get(&self, id: SourceId) -> Option<&(dyn Source + 'src)> {
        self.map.get(&id).map(|s| s.as_ref())
    }

    /// Like `HashMap::get_mut`.
    pub fn get_mut(&mut self, id: SourceId) -> Option<&mut (dyn Source + 'src)> {
        self.map.get_mut(&id).map(|s| s.as_mut())
    }

    /// Like `HashMap::insert`, but derives the `SourceId` key from the `Source`.
    pub fn insert(&mut self, source: Box<dyn Source + 'src>) {
        let id = source.source_id();
        self.map.insert(id, source);
    }

    /// Like `HashMap::len`.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Like `HashMap::iter_mut`.
    pub fn sources_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = (&'a SourceId, &'a mut (dyn Source + 'src))> {
        self.map.iter_mut().map(|(a, b)| (a, &mut **b))
    }

    /// Merge the given map into self.
    pub fn add_source_map(&mut self, other: SourceMap<'src>) {
        for (key, value) in other.map {
            self.map.entry(key).or_insert(value);
        }
    }
}
