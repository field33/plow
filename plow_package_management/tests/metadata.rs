mod common;

use crate::common::tests_filepath;
use harriet::TurtleDocument;
use plow_package_management::edit::{
    AddDependency, EditOperation, RemoveDependency, UpdateDependency,
};
use plow_package_management::metadata::OntologyMetadata;
use plow_package_management::resolve::Dependency;
use plow_package_management::version::SemanticVersion;
use std::convert::TryFrom;

#[test]
fn metadata_with_dependency() {
    let file_name = "../../data/example_ontologies/core_change_tracking.ttl";
    let ontology = std::fs::read_to_string(&tests_filepath(file_name)).unwrap();
    let document = TurtleDocument::parse_full(&ontology).unwrap();

    let metadata = OntologyMetadata::try_from(&document).unwrap();
    assert_eq!(metadata.dependencies.len(), 1);
    let dep = &metadata.dependencies[0];
    assert_eq!(dep.full_name, "@other/other_ontology");
    assert_eq!(dep.version_requirement, "=0.1.2");
}

#[test]
fn add_dependency() {
    let file_name = "../../data/example_ontologies/core_change_tracking.ttl";
    let ontology = std::fs::read_to_string(&tests_filepath(file_name)).unwrap();
    let mut document = TurtleDocument::parse_full(&ontology).unwrap();

    let add_operation = AddDependency {
        ontology_iri: "http://field33.com/ontologies/core_change_tracking/".to_string(),
        dependency: Dependency::<SemanticVersion>::try_from("@new/new_dependency =0.1.2").unwrap(),

        dependency_ontology_iri: "http://field33.com/ontologies/@new/new_dependency/".to_string(),
    };
    add_operation.apply(&mut document).unwrap();

    let metadata = OntologyMetadata::try_from(&document).unwrap();
    assert_eq!(metadata.dependencies.len(), 2);
}

#[test]
fn remove_dependency() {
    let file_name = "../../data/example_ontologies/core_change_tracking.ttl";
    let ontology = std::fs::read_to_string(&tests_filepath(file_name)).unwrap();
    let mut document = TurtleDocument::parse_full(&ontology).unwrap();

    let remove_operation = RemoveDependency {
        ontology_iri: "http://field33.com/ontologies/core_change_tracking/".to_string(),
        dependency_name: "@other/other_ontology".to_string(),
    };
    remove_operation.apply(&mut document).unwrap();

    let metadata = OntologyMetadata::try_from(&document).unwrap();
    assert_eq!(metadata.dependencies.len(), 0);
}

#[test]
fn update_dependency() {
    let file_name = "../../data/example_ontologies/core_change_tracking.ttl";
    let ontology = std::fs::read_to_string(&tests_filepath(file_name)).unwrap();
    let mut document = TurtleDocument::parse_full(&ontology).unwrap();

    let update_operation = UpdateDependency {
        ontology_iri: "http://field33.com/ontologies/core_change_tracking/".to_string(),
        dependency: Dependency::<SemanticVersion>::try_from("@other/other_ontology =20.0.0")
            .unwrap(),
        dependency_ontology_iri: "http://field33.com/ontologies/@other/other_ontology/".to_string(),
    };
    update_operation.apply(&mut document).unwrap();

    let metadata = OntologyMetadata::try_from(&document).unwrap();
    assert_eq!(metadata.dependencies.len(), 1);
    let dep = &metadata.dependencies[0];
    assert_eq!(dep.full_name, "@other/other_ontology");
    assert_eq!(dep.version_requirement, "=20.0.0");
}
