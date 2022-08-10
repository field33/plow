use crate::{
    config::{self, PlowConfigFile},
    feedback::{command_failed, info},
};
use anyhow::{anyhow, Result};

use colored::Colorize;
use harriet::{Item, Literal, Object, ParseError, Triples, TurtleDocument, IRI};
use plow_package_management::{
    package::{FieldMetadata, OrganizationToResolveFor},
    resolve::Dependency,
    version::SemanticVersion,
};
use std::{collections::HashMap, path::PathBuf};

fn dig_files(vec: &mut Vec<PathBuf>, path: PathBuf) -> std::io::Result<()> {
    if path.is_dir() {
        let paths = std::fs::read_dir(&path)?;
        for path_result in paths {
            let full_path = path_result?.path();
            dig_files(vec, full_path).unwrap();
        }
    } else {
        vec.push(path);
    }
    Ok(())
}

fn get_string_literal_from_object(object: &Object) -> Result<String> {
    match object {
        Object::Literal(literal) => match literal {
            Literal::RDFLiteral(rdf_literal) => {
                let turtle_string = &rdf_literal.string;
                Ok(turtle_string.to_string())
            }
            Literal::BooleanLiteral(_) => anyhow::bail!("Boolean literal found in RDF literal"),
        },

        _ => anyhow::bail!("Boolean literal found in RDF literal"),
    }
}

pub fn list_files<T: Into<PathBuf>>(
    path: T,
    extension_filter: &str,
) -> std::vec::Vec<std::path::PathBuf> {
    let mut vec = Vec::new();
    let path = path.into();
    dig_files(&mut vec, path).unwrap();
    vec.iter()
        .filter(|path| {
            if let Some(extension) = path.extension() {
                extension == extension_filter
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

fn extract_field_namespace(
    extracted_annotations: &HashMap<String, Result<Vec<String>, anyhow::Error>>,
) -> Option<String> {
    if let Some(Ok(literals)) = extracted_annotations.get("registry:packageName") {
        if let Some(value) = literals.first() {
            return value.split('/').map(std::borrow::ToOwned::to_owned).next();
        }
        return None;
    }
    None
}
fn extract_field_name(
    extracted_annotations: &HashMap<String, Result<Vec<String>, anyhow::Error>>,
) -> Option<String> {
    if let Some(Ok(literals)) = extracted_annotations.get("registry:packageName") {
        if let Some(value) = literals.first() {
            return value
                .split('/')
                .map(std::borrow::ToOwned::to_owned)
                .rev()
                .next();
        }
        return None;
    }
    None
}
fn extract_field_version(
    extracted_annotations: &HashMap<String, Result<Vec<String>, anyhow::Error>>,
) -> Option<String> {
    if let Some(Ok(literals)) = extracted_annotations.get("registry:packageVersion") {
        if let Some(value) = literals.first() {
            return Some(value.clone());
        }
        return None;
    }
    None
}

#[allow(clippy::too_many_lines)]
fn get_a_list_of_requested_dependencies_from_a_field(path_to_field: &str) -> Result<FieldMetadata> {
    let field_contents = std::fs::read_to_string(path_to_field).unwrap();
    let items = TurtleDocument::parse_full(&field_contents).unwrap().items;
    let mut ontology_iri = None;
    // File name -> (prefixed_name -> values as vec of string)
    let mut prefixed_name_to_values_in_ttl: HashMap<String, Result<Vec<String>, _>> =
        HashMap::new();

    let items = TurtleDocument::parse_full(&field_contents)
        .map_err(|err| match err {
            ParseError::ParseError(nom_err) => {
                anyhow::anyhow!("{}", nom_err.to_string())
            }
            ParseError::NotFullyParsed(message) => {
                anyhow::anyhow!("{}", message)
            }
        })?
        .items;
    for item in items {
        match item {
            Item::Statement(harriet::Statement::Triples(Triples::Labeled(
                subject,
                predicate_object_list,
            ))) => {
                for (iri, object_list) in predicate_object_list.list {
                    if let harriet::Subject::IRI(IRI::IRIReference(ref subject_iri)) = subject {
                        if let Some(base_iri) = &ontology_iri {
                            if subject_iri.iri.as_ref() == base_iri {
                                match iri {
                                    IRI::PrefixedName(prefixed_name) => {
                                        let prefixed_name = format!(
                                            "{}:{}",
                                            prefixed_name.prefix.unwrap_or_else(|| {
                                                std::borrow::Cow::from("".to_owned())
                                            }),
                                            prefixed_name.name.unwrap_or_else(|| {
                                                std::borrow::Cow::from("".to_owned())
                                            })
                                        );

                                        // Only get necessary fields from the ttl related to manifest
                                        match prefixed_name.as_str() {
                                            "registry:dependency"
                                            | "registry:packageName"
                                            | "registry:packageVersion" => {
                                                let collected_strings = object_list
                                                    .list
                                                    .iter()
                                                    .map(|object| {
                                                        get_string_literal_from_object(object)
                                                    })
                                                    .collect::<Result<Vec<String>, _>>();

                                                if prefixed_name == "rdfs:comment"
                                                    && prefixed_name_to_values_in_ttl
                                                        .get("rdfs:comment")
                                                        .is_some()
                                                {
                                                    continue;
                                                }
                                                if prefixed_name == "rdfs:label"
                                                    && prefixed_name_to_values_in_ttl
                                                        .get("rdfs:label")
                                                        .is_some()
                                                {
                                                    continue;
                                                }
                                                // Fill map
                                                prefixed_name_to_values_in_ttl
                                                    .insert(prefixed_name, collected_strings);
                                            }
                                            _ => {
                                                // Ignore
                                                continue;
                                            }
                                        }
                                    }
                                    IRI::IRIReference(_) => {
                                        // Ignore
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Item::Statement(harriet::Statement::Directive(harriet::Directive::Base(base))) => {
                ontology_iri = Some(base.iri.iri.to_string());
            }
            _ => {
                // Ignore
                continue;
            }
        }
    }

    let namespace = extract_field_namespace(&prefixed_name_to_values_in_ttl).ok_or_else(|| {
        anyhow!("registry:packageName could not be found or malformed in manifest.")
    })?;
    let name = extract_field_name(&prefixed_name_to_values_in_ttl).ok_or_else(|| {
        anyhow!("registry:packageName could not be found or malformed in manifest.")
    })?;
    let version = extract_field_version(&prefixed_name_to_values_in_ttl).ok_or_else(|| {
        anyhow!("registry:packageVersion could not be found or malformed in manifest.")
    })?;

    let dependencies =
        if let Some(Ok(deps)) = prefixed_name_to_values_in_ttl.get("registry:dependency") {
            deps.iter()
                .map(|dep_literal| {
                    Dependency::<SemanticVersion>::try_from(dep_literal.as_str()).unwrap()
                })
                .collect()
        } else {
            vec![]
        };

    Ok(FieldMetadata::new(namespace, name, version, dependencies))
}

pub fn prepare() -> Result<()> {
    let path_to_plow_toml = camino::Utf8PathBuf::from("./Plow.toml");
    let path_to_fields_dir = camino::Utf8PathBuf::from("./fields");
    let existing_field_paths_in_directory = list_files(".", "ttl");
    if existing_field_paths_in_directory.is_empty() && !path_to_fields_dir.exists() {
        command_failed("please run this command in a directory containing .ttl files in any depth");
    }
    let found_field_metadata = existing_field_paths_in_directory
        .iter()
        .map(|p| get_a_list_of_requested_dependencies_from_a_field(&p.to_string_lossy()))
        .collect::<Vec<Result<FieldMetadata, _>>>();

    // Create fields directory if it does not exist.
    if !path_to_fields_dir.exists() {
        std::fs::create_dir(&path_to_fields_dir);
        for (existing_path, field_metadata) in existing_field_paths_in_directory
            .iter()
            .zip(found_field_metadata.iter())
        {
            if let Ok(field_metadata) = field_metadata {
                std::fs::create_dir(path_to_fields_dir.join(&field_metadata.namespace));
                std::fs::create_dir(
                    path_to_fields_dir
                        .join(&field_metadata.namespace)
                        .join(&field_metadata.name),
                );
                let new_field_dest = path_to_fields_dir
                    .join(&field_metadata.namespace)
                    .join(&field_metadata.name)
                    .join(
                        &existing_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    );
                std::fs::copy(existing_path, &new_field_dest)?;
            } else {
                command_failed(&format!(
                    "Please complete the manifest of the field at {}",
                    existing_path.to_string_lossy()
                ));
            }
        }
    }

    let field_paths_in_fields_dir = list_files(path_to_fields_dir, "ttl");

    #[allow(clippy::unwrap_used)]
    let found_field_metadata: Vec<FieldMetadata> = field_paths_in_fields_dir
        .iter()
        .map(|p| get_a_list_of_requested_dependencies_from_a_field(&p.to_string_lossy()).unwrap())
        .collect();

    let workspace: config::Workspace = field_paths_in_fields_dir.into();

    #[allow(clippy::unwrap_used)]
    let config_file =
        toml::to_string::<PlowConfigFile>(&config::PlowConfigFile::with_workspace(&workspace))
            .unwrap();

    if let Err(err) = std::fs::write(path_to_plow_toml, config_file) {
        command_failed(&format!(
            "Plow failed to create Plow.toml in workspace. Details: {err}"
        ));
    }

    let _organizations_to_resolve_for = found_field_metadata
        .iter()
        .cloned()
        .map(std::convert::Into::into)
        .collect::<Vec<OrganizationToResolveFor>>();

    // Resolve deps and lock, prepare protege ws
    println!("\t{} is ready for plow.", "Workspace".green().bold());
    info("Workspace creation is a work in progress currently and not fully implemented. Expect errors.");
    Ok(())
}
