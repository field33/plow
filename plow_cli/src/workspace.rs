use crate::{
    config::{self, PlowConfigFile},
    feedback::command_failed,
};
use anyhow::Result;

use harriet::{Item, Literal, Object, Triples, TurtleDocument, IRI};
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

fn get_string_literal_from_object(object: &Object) -> String {
    match object {
        Object::Literal(literal) => match literal {
            Literal::RDFLiteral(rdf_literal) => {
                let turtle_string = &rdf_literal.string;
                turtle_string.to_string()
            }
            Literal::BooleanLiteral(_) => panic!(),
        },

        _ => panic!(),
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

fn get_a_list_of_requested_dependencies_from_a_field(path_to_field: &str) -> FieldMetadata {
    let mut base_iri = None;
    let mut prefixed_name_to_values_in_ttl: HashMap<String, Vec<String>> = HashMap::new();
    let field_contents = std::fs::read_to_string(path_to_field).unwrap();
    let items = TurtleDocument::parse_full(&field_contents).unwrap().items;
    for item in items {
        match item {
            Item::Statement(harriet::Statement::Triples(Triples::Labeled(
                subject,
                predicate_object_list,
            ))) => {
                for (iri, object_list) in predicate_object_list.list {
                    if let harriet::Subject::IRI(IRI::IRIReference(ref subject_iri)) = subject {
                        if let Some(base_iri) = &base_iri {
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

                                        // Only get necessary fields from the ttl
                                        match prefixed_name.as_str() {
                                            "registry:dependency"
                                            | "registry:packageName"
                                            | "registry:packageVersion" => {
                                                let collected_strings: Vec<String> = object_list
                                                    .list
                                                    .iter()
                                                    .map(|object| {
                                                        get_string_literal_from_object(object)
                                                    })
                                                    .collect();

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
                // save the base since we only need to parse statements for the ontology itself.
                base_iri = Some(base.iri.iri.to_string())
            }
            _ => {
                // Ignore
                continue;
            }
        }
    }

    let registry_package_name = &prefixed_name_to_values_in_ttl
        .get("registry:packageName")
        .unwrap()[0];
    let namespace_and_name: Vec<&str> = registry_package_name.split('/').collect();
    let namespace = namespace_and_name[0];
    let name = namespace_and_name[1];
    let version = &prefixed_name_to_values_in_ttl
        .get("registry:packageVersion")
        .unwrap()[0];

    let dependencies = if let Some(deps) = prefixed_name_to_values_in_ttl.get("registry:dependency")
    {
        deps.iter()
            .map(|dep_literal| {
                Dependency::<SemanticVersion>::try_from(dep_literal.as_str()).unwrap()
            })
            .collect()
    } else {
        vec![]
    };

    FieldMetadata::new(
        namespace.to_string(),
        name.to_string(),
        version.to_string(),
        dependencies,
    )
}

pub fn prepare() -> Result<()> {
    let path_to_plow_toml = camino::Utf8PathBuf::from("./Plow.toml");
    let path_to_fields_dir = camino::Utf8PathBuf::from("./fields");
    let existing_field_paths_in_directory = list_files(".", "ttl");
    if existing_field_paths_in_directory.is_empty() && !path_to_fields_dir.exists() {
        command_failed("please run this command in a directory containing .ttl files in any depth");
    }
    let found_field_metadata: Vec<FieldMetadata> = existing_field_paths_in_directory
        .iter()
        .map(|p| get_a_list_of_requested_dependencies_from_a_field(&p.to_string_lossy()))
        .collect();

    // Create fields directory if it does not exist.
    if !path_to_fields_dir.exists() {
        std::fs::create_dir(&path_to_fields_dir);
        for (existing_path, field_metadata) in existing_field_paths_in_directory
            .iter()
            .zip(found_field_metadata.iter())
        {
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
        }
    }

    let field_paths_in_fields_dir = list_files(path_to_fields_dir, "ttl");

    let found_field_metadata: Vec<FieldMetadata> = field_paths_in_fields_dir
        .iter()
        .map(|p| get_a_list_of_requested_dependencies_from_a_field(&p.to_string_lossy()))
        .collect();

    let workspace: config::Workspace = field_paths_in_fields_dir.into();
    let config_file =
        toml::to_string::<PlowConfigFile>(&config::PlowConfigFile::with_workspace(&workspace))
            .unwrap();

    std::fs::write(path_to_plow_toml, config_file);

    let organizations_to_resolve_for = found_field_metadata
        .iter()
        .cloned()
        .map(|meta| meta.into())
        .collect::<Vec<OrganizationToResolveFor>>();

    // Resolve deps and lock, prepare protege ws

    todo!()
}
