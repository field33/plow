mod source_id;

use std::collections::hash_map::HashMap;
use std::fmt;
use std::task::Poll;

pub use self::source_id::SourceId;

/// Something that finds and downloads remote packages based on names and versions.
pub trait Source {
    /// Returns the `SourceId` corresponding to this source.
    fn source_id(&self) -> SourceId;

    /// Returns the replaced `SourceId` corresponding to this source.
    fn replaced_source_id(&self) -> SourceId {
        self.source_id()
    }

    /// Attempts to find the packages that match a dependency request.
    fn query(&mut self, dep: &Dependency, f: &mut dyn FnMut(Summary)) -> Poll<Result<()>>;

    fn query_vec(&mut self, dep: &Dependency) -> Poll<Result<Vec<Summary>>> {
        let mut ret = Vec::new();
        self.query(dep, &mut |s| ret.push(s)).map_ok(|_| ret)
    }

    /// Ensure that the source is fully up-to-date for the current session on the next query.
    fn invalidate_cache(&mut self);

    /// Fetches the full package for each name and version specified.
    fn download(&mut self, package: PackageId) -> Result<MaybePackage>;

    /// Generates a unique string which represents the fingerprint of the
    /// current state of the source.
    ///
    /// This fingerprint is used to determine the "fresheness" of the source
    /// later on. It must be guaranteed that the fingerprint of a source is
    /// constant if and only if the output product will remain constant.
    ///
    /// The `pkg` argument is the package which this fingerprint should only be
    /// interested in for when this source may contain multiple packages.
    fn fingerprint(&self, pkg: &Package) -> Result<String>;

    /// Describes this source in a human readable fashion, used for display in
    /// resolver error messages currently.
    fn describe(&self) -> String;

    /// Returns whether a source is being replaced by another here.
    fn is_replaced(&self) -> bool {
        false
    }

    /// Query if a package is yanked. Only registry sources can mark packages
    /// as yanked. This ignores the yanked whitelist.
    fn is_yanked(&mut self, _pkg: PackageId) -> Poll<Result<bool>>;

    // TODO:
    /// Block until all outstanding Poll::Pending requests are `Poll::Ready`.
    ///
    /// After calling this function, the source should return `Poll::Ready` for
    /// any queries that previously returned `Poll::Pending`.
    ///
    /// If no queries previously returned `Poll::Pending`, and `invalidate_cache`
    /// was not called, this function should be a no-op.
    fn block_until_ready(&mut self) -> Result<()>;
}

pub enum MaybePackage {
    Ready(Package),
    Download { url: String, descriptor: String },
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
    fn query(&mut self, dep: &Dependency, f: &mut dyn FnMut(Summary)) -> Poll<Result<()>> {
        (**self).query(dep, f)
    }

    fn invalidate_cache(&mut self) {
        (**self).invalidate_cache()
    }

    /// Forwards to `Source::download`.
    fn download(&mut self, id: PackageId) -> Result<MaybePackage> {
        (**self).download(id)
    }

    /// Forwards to `Source::fingerprint`.
    fn fingerprint(&self, pkg: &Package) -> Result<String> {
        (**self).fingerprint(pkg)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }

    fn is_replaced(&self) -> bool {
        (**self).is_replaced()
    }

    fn is_yanked(&mut self, pkg: PackageId) -> Poll<Result<bool>> {
        (**self).is_yanked(pkg)
    }

    fn block_until_ready(&mut self) -> Result<()> {
        (**self).block_until_ready()
    }
}

impl<'a, T: Source + ?Sized + 'a> Source for &'a mut T {
    fn source_id(&self) -> SourceId {
        (**self).source_id()
    }

    fn replaced_source_id(&self) -> SourceId {
        (**self).replaced_source_id()
    }

    fn query(&mut self, dep: &Dependency, f: &mut dyn FnMut(Summary)) -> Poll<Result<()>> {
        (**self).query(dep, f)
    }

    fn invalidate_cache(&mut self) {
        (**self).invalidate_cache()
    }

    fn download(&mut self, id: PackageId) -> Result<MaybePackage> {
        (**self).download(id)
    }

    fn fingerprint(&self, pkg: &Package) -> Result<String> {
        (**self).fingerprint(pkg)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }

    fn is_replaced(&self) -> bool {
        (**self).is_replaced()
    }

    fn is_yanked(&mut self, pkg: PackageId) -> Poll<Result<bool>> {
        (**self).is_yanked(pkg)
    }

    fn block_until_ready(&mut self) -> Result<()> {
        (**self).block_until_ready()
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
