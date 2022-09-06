use crate::{
    lock::LockFile,
    metadata::OntologyMetadata,
    package::{RetrievedPackageSet, RetrievedPackageVersion},
    registry::Registry,
};
use anyhow::anyhow;
use camino::{Utf8Path, Utf8PathBuf};
use harriet::TurtleDocument;

use sha2::{Digest, Sha256};
use std::{
    fs::{create_dir_all, hard_link, read_to_string, File},
    io::Write,
    os::unix::ffi::OsStrExt,
};

/// The name of the main directory in the platform-defined documents directory where we place our data.
const TOOLS_MAIN_DIR: &str = "ontology_tools";
/// The name of the subdirectory below the main directory where we place our workspaces.
const WORKSPACES_SUBDIR: &str = "workspaces";
/// The name of the subdirectory below the specific workspace where we place our dependency.
const WORKSPACE_DEPS_SUBDIR: &str = "deps";

/// Fixed filename for the catalog file that Protege reads to resolve imports.
///
/// See <https://protegewiki.stanford.edu/wiki/Importing_Ontologies_in_P41>.
const PROTEGE_CATALOG_FILE_FILENAME: &str = "catalog-v001.xml";

#[derive(Debug, Clone)]
pub struct OntologyWorkspace {
    pub workspace_dir: Utf8PathBuf,
    pub ontology_file: Utf8PathBuf,
}

impl OntologyWorkspace {
    /// Creates a directory in our workspace directory based on the provided file.
    ///
    /// The workspace directory will contain a symlink to the original file (so changes propagate
    /// back to the original file), and can also be used to place e.g. a catalog file and resolved
    /// dependencies for the file.
    pub fn mirror_file_to_workspace(
        original_ontology_path: &Utf8Path,
    ) -> Result<Self, anyhow::Error> {
        let mut sha256 = Sha256::new();
        sha256.update(original_ontology_path.as_os_str().as_bytes());
        let original_path_hash: String = format!("{:X}", sha256.finalize());
        let workspace_dir = Utf8PathBuf::from_path_buf(
            dirs::document_dir()
                .ok_or_else(|| anyhow!("No document dir known for platform"))?
                .join(TOOLS_MAIN_DIR)
                .join(WORKSPACES_SUBDIR)
                .join(original_path_hash),
        )
        .expect("Documents directory is not a valid path.");
        let symlinked_ontology_path = workspace_dir.join(
            original_ontology_path
                .file_name()
                .ok_or_else(|| anyhow!("Provided path is not a file"))?,
        );

        if !workspace_dir.exists() {
            std::fs::create_dir_all(&workspace_dir)?;
        }
        if !symlinked_ontology_path.exists() {
            hard_link(&original_ontology_path, &symlinked_ontology_path)?;
        }

        Ok(Self {
            workspace_dir,
            ontology_file: symlinked_ontology_path,
        })
    }

    /// Locks the dependencies of the workspace.
    pub fn lock(
        self,
        registry: &dyn Registry,
        workspace_root: Option<Utf8PathBuf>,
    ) -> Result<OntologyWorkspaceLocked, anyhow::Error> {
        let contents_str = read_to_string(&self.ontology_file)?;
        let document = TurtleDocument::parse_full(&contents_str)
            .map_err(|_| anyhow!("Unable to parse dependency contents as ontology"))?;
        let ontology_metadata = OntologyMetadata::try_from(&document)?;

        Ok(OntologyWorkspaceLocked {
            workspace_dir: self.workspace_dir,
            ontology_file: self.ontology_file,
            lockfile: LockFile::lock_with_registry(
                ontology_metadata.into(),
                registry,
                workspace_root,
            )?,
        })
    }

    pub const fn ontology_file(&self) -> &Utf8PathBuf {
        &self.ontology_file
    }
}

#[derive(Debug, Clone)]
pub struct OntologyWorkspaceLocked {
    workspace_dir: Utf8PathBuf,
    ontology_file: Utf8PathBuf,
    lockfile: LockFile,
}

impl OntologyWorkspaceLocked {
    pub fn retrieve_dependencies(
        self,
        registry: &dyn Registry,
    ) -> Result<OntologyWorkspaceWithRetrievedDeps, anyhow::Error> {
        let dependency_dir = self.workspace_dir.join(WORKSPACE_DEPS_SUBDIR);
        if !dependency_dir.exists() {
            create_dir_all(&dependency_dir)?;
        }

        let mut retrieved_deps = vec![];
        for dep in &self.lockfile.locked_dependencies.packages {
            let contents = registry.retrieve_package(dep)?;
            let contents_str = String::from_utf8(contents)?;

            let document = TurtleDocument::parse_full(&contents_str)
                .map_err(|_| anyhow!("Unable to parse dependency contents as ontology"))?;
            let metadata = OntologyMetadata::try_from(&document)?;

            let dependency_file_path = dependency_dir
                .clone()
                .join(format!("{}.ttl", metadata.canonical_prefix));
            let mut file = File::create(&dependency_file_path)?;
            file.write_all(contents_str.as_bytes())?;

            retrieved_deps.push(RetrievedPackageVersion {
                ontology_iri: metadata.root_prefix,
                package: dep.clone(),
                file_path: dependency_file_path,
            });
        }

        Ok(OntologyWorkspaceWithRetrievedDeps {
            workspace_dir: self.workspace_dir,
            ontology_file: self.ontology_file,
            lockfile: self.lockfile,
            dependencies: RetrievedPackageSet {
                packages: retrieved_deps,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct OntologyWorkspaceWithRetrievedDeps {
    workspace_dir: Utf8PathBuf,
    ontology_file: Utf8PathBuf,
    lockfile: LockFile,
    dependencies: RetrievedPackageSet,
}

impl OntologyWorkspaceWithRetrievedDeps {
    pub fn retrieved_dependencies(&self) -> &[RetrievedPackageVersion] {
        self.dependencies.packages.as_slice()
    }
    pub fn generate_catalog_file(self) -> Result<OntologyWorkspaceWithCatalog, anyhow::Error> {
        Ok(OntologyWorkspaceWithCatalog {
            catalog_file: CatalogFile::generate_catalog_file(
                &self.workspace_dir,
                &self.dependencies,
            )?,
            workspace_dir: self.workspace_dir,
            ontology_file: self.ontology_file,
            lockfile: self.lockfile,
            dependencies: self.dependencies,
        })
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct OntologyWorkspaceWithCatalog {
    workspace_dir: Utf8PathBuf,
    ontology_file: Utf8PathBuf,
    lockfile: LockFile,
    dependencies: RetrievedPackageSet,
    catalog_file: CatalogFile,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CatalogFile {
    path: Utf8PathBuf,
}

impl CatalogFile {
    const CATALOG_FILE_CONTENT_START: &'static str = r#"
    <?xml version="1.0" encoding="UTF-8" standalone="no"?>
    <catalog prefer="public" xmlns="urn:oasis:names:tc:entity:xmlns:xml:catalog">
    <group id="Folder Repository, directory=, recursive=true, Auto-Update=true, version=2" prefer="public" xml:base=""/>
"#;

    const CATALOG_FILE_CONTENT_END: &'static str = r#"
    </catalog>
"#;

    const CATALOG_FILE_SOURCE_NOTE: &'static str = "Added via ontology_tools";

    pub fn generate_catalog_file(
        workspace_dir: &Utf8Path,
        package_set: &RetrievedPackageSet,
    ) -> Result<Self, anyhow::Error> {
        let catalog_file_path = workspace_dir.join(PROTEGE_CATALOG_FILE_FILENAME);

        let mut contents = String::new();
        contents.push_str(Self::CATALOG_FILE_CONTENT_START.trim());
        for package in &package_set.packages {
            contents.push_str(&format!(
                r#"<uri id="{source_note}" name="{ontology_iri}" uri={file_path:?}/>"#,
                source_note = Self::CATALOG_FILE_SOURCE_NOTE,
                ontology_iri = package.ontology_iri,
                file_path = package.file_path,
            ));
            contents.push('\n');
        }
        contents.push_str(Self::CATALOG_FILE_CONTENT_END.trim());

        let mut file = File::create(&catalog_file_path)?;
        file.write_all(contents.as_bytes())?;

        Ok(Self {
            path: catalog_file_path,
        })
    }
}
