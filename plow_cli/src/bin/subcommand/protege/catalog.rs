use camino::{Utf8Path, Utf8PathBuf};
use plow_package_management::package::RetrievedPackageSet;

/// Fixed filename for the catalog file that Protege reads to resolve imports.
///
/// See <https://protegewiki.stanford.edu/wiki/Importing_Ontologies_in_P41>.
const PROTEGE_CATALOG_FILE_FILENAME: &str = "catalog-v001.xml";

const CATALOG_FILE_CONTENT_START: &str = r#"
    <?xml version="1.0" encoding="UTF-8" standalone="no"?>
    <catalog prefer="public" xmlns="urn:oasis:names:tc:entity:xmlns:xml:catalog">
    <group id="Folder Repository, directory=, recursive=true, Auto-Update=true, version=2" prefer="public" xml:base=""/>
"#;

const CATALOG_FILE_CONTENT_END: &str = r#"
    </catalog>
"#;

const CATALOG_FILE_SOURCE_NOTE: &str = "Added via plow";

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CatalogFile {
    path: Utf8PathBuf,
}

impl CatalogFile {
    pub fn generate(
        workspace_dir: &Utf8Path,
        // vec of resolved dependencies
        package_set: &RetrievedPackageSet,
    ) -> Result<Self, anyhow::Error> {
        let catalog_file_path = workspace_dir.join(PROTEGE_CATALOG_FILE_FILENAME);

        let mut contents = String::new();
        contents.push_str(CATALOG_FILE_CONTENT_START.trim());
        for package in &package_set.packages {
            contents.push_str(&format!(
                r#"<uri id="{source_note}" name="{ontology_iri}" uri={file_path:?}/>"#,
                source_note = CATALOG_FILE_SOURCE_NOTE,
                ontology_iri = package.ontology_iri,
                file_path = package.file_path,
            ));
            contents.push('\n');
        }
        contents.push_str(CATALOG_FILE_CONTENT_END.trim());
        std::fs::write(&catalog_file_path, contents.as_bytes())?;

        Ok(Self {
            path: catalog_file_path,
        })
    }
}
