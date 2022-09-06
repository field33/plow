#![allow(clippy::restriction, clippy::useless_vec)]
#![allow(unused_assignments)]
use camino::Utf8PathBuf;
use plow_package_management::{
    lock::LOCK_FILE_NAME, registry::on_disk::OnDiskRegistry, workspace::OntologyWorkspace,
};
use std::path::PathBuf;

#[allow(dead_code)]
#[ignore = "Creates problems with Bazel: The test writes a file but there are privilege issues when running under Bazel. Could be fixed later."]
fn lock_read_write() {
    let mut data_dir = Utf8PathBuf::from("../data");

    let selected_file_path = data_dir
        .join("example_ontologies")
        .join("software_development")
        .join("software_development.ttl");

    if selected_file_path
        .parent()
        .unwrap()
        .join(LOCK_FILE_NAME)
        .exists()
    {
        // remove the lock file
        std::fs::remove_file(selected_file_path.parent().unwrap().join(LOCK_FILE_NAME)).unwrap();
    }

    let registry_path = data_dir
        .join("example_registries")
        .join("partial_staging_registry");

    let workspace = OntologyWorkspace {
        workspace_dir: selected_file_path.parent().unwrap().to_path_buf(),
        ontology_file: selected_file_path.clone(),
    };

    let registry = OnDiskRegistry::new(registry_path).unwrap();

    let registry = Box::new(registry);

    let mut retrieved_from_resolution = vec![];
    let mut retrieved_from_lockfile = vec![];

    // Write the lock file.
    match workspace
        .clone()
        .lock(registry.as_ref(), Some(selected_file_path.clone()))
    {
        Ok(locked) => {
            retrieved_from_resolution = locked
                .retrieve_dependencies(registry.as_ref())
                .unwrap()
                .retrieved_dependencies()
                .to_vec();
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }

    // Resolve from previously written lock file
    match workspace.lock(registry.as_ref(), Some(selected_file_path)) {
        Ok(locked) => {
            retrieved_from_lockfile = locked
                .retrieve_dependencies(registry.as_ref())
                .unwrap()
                .retrieved_dependencies()
                .to_vec();
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }

    // Check if they are the same
    retrieved_from_resolution
        .iter()
        .zip(retrieved_from_lockfile)
        .for_each(|(a, b)| {
            assert_eq!(a.package.package_name, b.package.package_name);
            assert_eq!(a.package.version, b.package.version);
        });
}
