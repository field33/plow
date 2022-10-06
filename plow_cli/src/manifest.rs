#![allow(dead_code)]

mod macros;
mod utils;

use anyhow::{anyhow, Result};
use camino::Utf8Path;
use harriet::{
    Literal, Object, ObjectList, ParseError, Statement, Triples, TurtleDocument, Verb, Whitespace,
    IRI,
};
use lazy_static::lazy_static;
use libplow::registry::Dependency;
use libplow::registry::SemanticVersion;
use regex::Regex;
use serde_json::map;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;

use crate::extract_mandatory_annotation_from;
use crate::extract_optional_string_annotation_from;

use self::utils::get_string_literal_from_object;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldAuthor {
    name: String,
    email: String,
}

impl core::fmt::Display for FieldAuthor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

#[derive(Debug)]
struct ManifestMetadata {
    full_name: String,
    namespace: String,
    name: String,
    version: SemanticVersion,
    authors: Vec<FieldAuthor>,
    license: Option<String>,
    license_spdx: Option<String>,
    description: String,
    homepage: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    keywords: Vec<String>,
    ontology_iri: Option<String>,
    dependencies: Vec<Dependency<SemanticVersion>>,
    categories: Vec<String>,
    title: String,
    short_description: String,
}

// TODO: Implement license retrieval and validation when necessary.

#[derive(Debug)]
pub struct FieldManifest<'manifest> {
    field_cksum: String,
    statements: Vec<Statement<'manifest>>,
    metadata: ManifestMetadata,
}

impl<'manifest> FieldManifest<'manifest> {
    pub fn quick_extract_field_full_name<P: AsRef<Utf8Path>>(field_path: &P) -> Result<String> {
        self::utils::quick_extract_field_full_name(field_path)
    }

    pub fn full_name(&self) -> &str {
        &self.metadata.full_name
    }

    pub fn namespace_and_name(&self) -> (&str, &str) {
        (&self.metadata.namespace, &self.metadata.name)
    }

    pub fn version(&self) -> &SemanticVersion {
        &self.metadata.version
    }

    pub fn ontology_iri(&self) -> Option<&str> {
        self.metadata.ontology_iri.as_ref().map(|s| s.as_str())
    }

    pub fn authors(&self) -> &[FieldAuthor] {
        &self.metadata.authors
    }

    pub fn description(&self) -> &str {
        &self.metadata.description
    }

    pub fn short_description(&self) -> &str {
        &self.metadata.short_description
    }

    pub fn homepage(&self) -> Option<&str> {
        self.metadata.homepage.as_ref().map(|s| s.as_str())
    }

    pub fn repository(&self) -> Option<&str> {
        self.metadata.repository.as_ref().map(|s| s.as_str())
    }

    pub fn documentation(&self) -> Option<&str> {
        self.metadata.documentation.as_ref().map(|s| s.as_str())
    }

    pub fn keywords(&self) -> &[String] {
        &self.metadata.keywords
    }

    pub fn keywords_comma_separated(&self) -> String {
        self.metadata.keywords.join(", ")
    }

    pub fn categories(&self) -> &[String] {
        &self.metadata.categories
    }

    pub fn title(&self) -> &str {
        &self.metadata.title
    }

    pub fn dependencies(&self) -> &[Dependency<SemanticVersion>] {
        &self.metadata.dependencies
    }

    pub fn field_hash(&self) -> Result<String> {
        let field_hash = libplow::utils::hash_field(
            &self.metadata.namespace,
            &self.metadata.name,
            &self.metadata.version.to_string(),
        );
        Ok(field_hash)
    }
}

impl<'manifest> FieldManifest<'manifest> {
    #[allow(clippy::too_many_lines)]
    pub fn new(field_contents: &'manifest str) -> Result<Self> {
        let mut ontology_iri = None;
        // File name -> (prefixed_name -> values as vec of string)
        let mut prefixed_name_to_values_in_ttl: HashMap<String, Result<Vec<String>, _>> =
            HashMap::new();

        let statements = TurtleDocument::parse_full(field_contents)
            .map_err(|err| match err {
                ParseError::ParseError(nom_err) => {
                    anyhow::anyhow!("{}", nom_err.to_string())
                }
                ParseError::NotFullyParsed(message) => {
                    anyhow::anyhow!("{}", message)
                }
            })?
            .statements;

        for statement in &statements {
            match statement {
                harriet::Statement::Triples(Triples::Labeled(
                    _,
                    subject,
                    predicate_object_list,
                )) => {
                    for (_, verb, object_list, _) in &predicate_object_list.list {
                        if let harriet::Subject::IRI(IRI::IRIReference(ref subject_iri)) = subject {
                            if let Some(base_iri) = &ontology_iri {
                                if subject_iri.iri.as_ref() == base_iri {
                                    match verb {
                                        Verb::IRI(IRI::PrefixedName(prefixed_name)) => {
                                            let prefixed_name = format!(
                                                "{}:{}",
                                                prefixed_name.prefix.as_ref().unwrap(),
                                                prefixed_name.name.as_ref().unwrap()
                                            );

                                            // Only get necessary fields from the ttl related to manifest
                                            match prefixed_name.as_str() {
                                                "owl:imports"
                                                | "registry:author"
                                                | "registry:category"
                                                | "registry:dependency"
                                                | "registry:keyword"
                                                | "registry:packageName"
                                                | "registry:packageVersion"
                                                | "registry:repository"
                                                | "registry:homepage"
                                                | "registry:documentation"
                                                | "registry:license"
                                                | "registry:licenseSPDX"
                                                | "registry:shortDescription"
                                                | "rdfs:comment"
                                                | "rdfs:label" => {
                                                    let collected_strings = object_list
                                                        .list
                                                        .iter()
                                                        .map(|(_, _, object)| {
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
                                        _ => {
                                            // Ignore
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                harriet::Statement::Directive(harriet::Directive::Base(base)) => {
                    ontology_iri = Some(base.iri.iri.to_string());
                }
                _ => {
                    // Ignore
                    continue;
                }
            }
        }

        let authors =
            extract_mandatory_annotation_from!("registry:author", prefixed_name_to_values_in_ttl)
                .iter()
                .map(|author_literal| {
                    let author_literal = author_literal.trim();
                    let (name, email) = author_literal.split_once('<').ok_or_else(|| {
                        anyhow!("Invalid author literal. Expected format: 'Name Surname <email>'")
                    })?;
                    let email = email.trim_end_matches('>');
                    Ok(FieldAuthor {
                        name: name.to_string(),
                        email: email.to_string(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;

        let categories =
            extract_mandatory_annotation_from!("registry:category", prefixed_name_to_values_in_ttl);

        let dependencies =
            if let Some(literals) = prefixed_name_to_values_in_ttl.get("registry:dependency") {
                let literals = literals.map_err(|err| {
                    anyhow::anyhow!(
                        "Error parsing registry:dependency in the manifest file. Details: {err}",
                    )
                })?;
                literals
                    .iter()
                    .map(|dependency| {
                        let (name, req) = dependency.split_once(' ').ok_or_else(|| {
                            anyhow!(
                    "Invalid dependency. Expected format: '@namespace/name <version-requirement>'"
                )
                        })?;
                        Ok(Dependency::<SemanticVersion>::try_new(name, req)?)
                    })
                    .collect::<Result<Vec<_>>>()?
            } else {
                vec![]
            };

        let keywords =
            extract_mandatory_annotation_from!("registry:keyword", prefixed_name_to_values_in_ttl);

        let full_name = extract_mandatory_annotation_from!(
            "registry:packageName",
            prefixed_name_to_values_in_ttl
        )
        .first()
        .ok_or_else(|| anyhow!("Missing registry:packageName"))?
        .to_string();

        let (namespace, name) =
            Dependency::<SemanticVersion>::split_and_validate_full_field_name(&full_name)?;

        let version = SemanticVersion::from_str(
            extract_mandatory_annotation_from!(
                "registry:packageVersion",
                prefixed_name_to_values_in_ttl
            )
            .first()
            .ok_or_else(|| anyhow!("Missing registry:packageVersion"))?,
        )?;

        let repository = extract_optional_string_annotation_from!(
            "registry:repository",
            prefixed_name_to_values_in_ttl
        );
        let homepage = extract_optional_string_annotation_from!(
            "registry:homepage",
            prefixed_name_to_values_in_ttl
        );

        let documentation = extract_optional_string_annotation_from!(
            "registry:documentation",
            prefixed_name_to_values_in_ttl
        );

        // TODO: Validate so these are not both missing
        let license = extract_optional_string_annotation_from!(
            "registry:license",
            prefixed_name_to_values_in_ttl
        );

        let license_spdx = extract_optional_string_annotation_from!(
            "registry:licenseSPDX",
            prefixed_name_to_values_in_ttl
        );
        //

        let short_description = extract_mandatory_annotation_from!(
            "registry:shortDescription",
            prefixed_name_to_values_in_ttl
        )
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("Missing registry:shortDescription"))?;

        let description =
            extract_mandatory_annotation_from!("registry:comment", prefixed_name_to_values_in_ttl)
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("Missing registry:comment"))?;

        let title =
            extract_mandatory_annotation_from!("registry:label", prefixed_name_to_values_in_ttl)
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("Missing registry:label"))?;

        let mut hasher = Sha256::new();
        hasher.update(field_contents.as_bytes());
        let field_cksum = format!("{:X}", hasher.finalize()).to_lowercase();

        let metadata = ManifestMetadata {
            full_name,
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            version,
            ontology_iri,
            authors,
            categories,
            dependencies,
            keywords,
            license,
            license_spdx,
            repository,
            homepage,
            documentation,
            short_description,
            description,
            title,
        };

        Ok(Self {
            field_cksum,
            metadata,
            statements,
        })
    }

    pub fn update_owl_imports_and_serialize(
        &self,
        new_object_list: ObjectList<'manifest>,
        mut statements: Vec<Statement<'manifest>>,
        target_path: &Utf8Path,
    ) {
        for statement in &mut statements {
            match statement {
                harriet::Statement::Triples(Triples::Labeled(
                    _,
                    subject,
                    predicate_object_list,
                )) => {
                    for (_, verb, object_list, _) in &mut predicate_object_list.list {
                        match verb {
                            Verb::IRI(IRI::PrefixedName(prefixed_name)) => {
                                let prefixed_name = format!(
                                    "{}:{}",
                                    prefixed_name.prefix.as_ref().unwrap(),
                                    prefixed_name.name.as_ref().unwrap(),
                                );

                                if prefixed_name == "owl:imports" {
                                    std::mem::replace(
                                        &mut object_list.list,
                                        new_object_list.list.clone(),
                                    );
                                    break;
                                }
                            }
                            _ => continue,
                        }
                    }
                }
                _ => {}
            }
        }

        let new_doc = TurtleDocument {
            statements: statements.clone(),
            trailing_whitespace: None,
        };
        std::fs::write(target_path, new_doc.to_string()).unwrap();
    }

    pub fn create_owl_imports_and_serialize(
        &self,
        new_predicate: (
            Whitespace<'manifest>,
            harriet::Verb<'manifest>,
            ObjectList<'manifest>,
            Option<Whitespace<'manifest>>,
        ),
        mut statements: Vec<Statement<'manifest>>,
        target_path: &Utf8Path,
    ) {
        for statement in &mut statements {
            match statement {
                harriet::Statement::Triples(Triples::Labeled(
                    _,
                    subject,
                    predicate_object_list,
                )) => {
                    predicate_object_list.list.push(new_predicate.clone());
                }
                _ => {}
            }
        }

        let new_doc = TurtleDocument {
            statements: statements.clone(),
            trailing_whitespace: None,
        };
        std::fs::write(target_path, new_doc.to_string()).unwrap();
    }

    pub fn dependencies_stated_in_owl_imports(
        &self,
    ) -> Result<(ObjectList, Vec<String>, String, Vec<Statement>), Vec<Statement>> {
        let mut base_iri: Option<String> = None;
        let stuff = self
            .statements
            .iter()
            .fold(vec![], |mut acc, statement| match statement {
                harriet::Statement::Triples(Triples::Labeled(
                    _,
                    subject,
                    predicate_object_list,
                )) => {
                    for (_, verb, object_list, _) in &predicate_object_list.list {
                        if let harriet::Subject::IRI(IRI::IRIReference(ref subject_iri)) = subject {
                            if let Some(base_iri) = &base_iri {
                                if subject_iri.iri.as_ref() == base_iri {
                                    match verb {
                                        Verb::IRI(IRI::PrefixedName(prefixed_name)) => {
                                            let prefixed_name = format!(
                                                "{}:{}",
                                                prefixed_name.prefix.as_ref().unwrap(),
                                                prefixed_name.name.as_ref().unwrap(),
                                            );

                                            if prefixed_name == "owl:imports" {
                                                let mut dep_names = vec![];
                                                let mut object_iris = vec![];
                                                for (_, _, object) in &object_list.list {
                                                    if let Object::IRI(IRI::IRIReference(
                                                        ref object_iri,
                                                    )) = object
                                                    {
                                                        object_iris.push(object_iri);
                                                    }
                                                }

                                                for object_iri in &object_iris {
                                                    let iri_literal =
                                                        object_iri.iri.as_ref().to_owned();
                                                    let iris = iri_literal
                                                        .split("/")
                                                        .collect::<Vec<&str>>();
                                                    let mut it = iris.iter().rev();
                                                    it.next();
                                                    let name = it.next().unwrap().to_owned();
                                                    let namespace = it.next().unwrap().to_owned();
                                                    let dep_name = format!("{namespace}/{name}");
                                                    dep_names.push(dep_name);
                                                }
                                                let base_iri_parts =
                                                    base_iri.split("/").collect::<Vec<&str>>();
                                                let base_iri_beginning = base_iri_parts
                                                    [..base_iri_parts.len() - 3]
                                                    .join("/");

                                                acc.push((
                                                    object_list.clone(),
                                                    dep_names,
                                                    base_iri_beginning,
                                                    self.statements.clone(),
                                                ));
                                            }
                                        }
                                        _ => continue,
                                    }
                                }
                                continue;
                            }
                            continue;
                        }
                        continue;
                    }
                    acc
                }
                harriet::Statement::Directive(harriet::Directive::Base(base)) => {
                    base_iri = Some(base.iri.iri.to_string());
                    acc
                }
                _ => acc,
            });
        stuff
            .get(0)
            .map_or_else(|| Err(self.statements.clone()), |x| Ok(x.clone()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::restriction)]

    use super::*;

    const VALID_FIELD: &str = r#"
@prefix : <http://field33.com/ontologies/@fld33/test/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xml: <http://www.w3.org/XML/1998/namespace> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix registry: <http://field33.com/ontologies/REGISTRY/> .
@base <http://field33.com/ontologies/@fld33/test/> .
<http://field33.com/ontologies/@fld33/test/> rdf:type owl:Ontology ;
owl:imports <http://field33.com/ontologies/@fld33/communication/> ,
<http://field33.com/ontologies/@fld33/organization/> ;
registry:canonicalPrefix "test" ;
registry:author "Miles Davis <miles@field33.com>" ,
"Joe Pass <joe@field33.com>" ;
registry:category "Communication" ,
"Organization" ,
"Upper Ontology";
registry:dependency "@fld33/communication ^0.1.0" ,
"@fld33/organization ^0.1.1" ;
registry:keyword "Communication" ,
"Field 33 Package" ,
"Organization" ,
"Upper Ontology";
registry:license "MIT License";
registry:licenseSPDX  "MIT";
registry:documentation  "https://field33.com";
registry:ontologyFormatVersion "v1" ;
registry:packageName "@fld33/test" ;
registry:packageVersion "0.1.2" ;
registry:repository "https://github.com/field33/ontology-workspace/tree/main/%40fld33/test" ;
registry:shortDescription "A short description of the field"@en ;
rdfs:comment "A long description of the field."@en ;
rdfs:label "My interesting field"@en .

registry:dependency rdf:type owl:AnnotationProperty .
    "#;

    #[test]
    fn manifest_getters_and_extraction() {
        let field_manifest = FieldManifest::new(VALID_FIELD).unwrap();
        let (namespace, name) = field_manifest.namespace_and_name();
        assert_eq!(namespace, "@fld33");
        assert_eq!(name, "test");
        assert_eq!(field_manifest.full_name(), "@fld33/test");
        assert_eq!(field_manifest.version(), &libplow::semver!("0.1.2"));
        assert_eq!(
            field_manifest.authors(),
            vec![
                FieldAuthor {
                    name: "Miles Davis".to_string(),
                    email: "miles@field33.com".to_string()
                },
                FieldAuthor {
                    name: "Joe Pass".to_string(),
                    email: "joe@field33.com".to_string(),
                }
            ]
        );
        let mut it = field_manifest.authors().iter();
        assert_eq!(
            it.next().unwrap().to_string(),
            "Miles Davis <miles@field33.com>"
        );
        assert_eq!(it.next().unwrap().to_string(), "Joe Pass <joe@field33.com>");

        assert_eq!(field_manifest.homepage(), None);
        assert_eq!(
            field_manifest.repository(),
            Some("https://github.com/field33/ontology-workspace/tree/main/%40fld33/test")
        );
        assert_eq!(field_manifest.documentation(), Some("https://field33.com"));

        // TODO:
        // assert_eq!(field_manifest.license(), Some("MIT License".to_owned()));
        // assert_eq!(field_manifest.license_spdx(), Some("MIT".to_owned()));
        assert_eq!(
            field_manifest.categories(),
            vec![
                "Communication".to_owned(),
                "Organization".to_owned(),
                "Upper Ontology".to_owned()
            ]
        );
        assert_eq!(
            field_manifest.keywords(),
            vec![
                "Communication".to_owned(),
                "Field 33 Package".to_owned(),
                "Organization".to_owned(),
                "Upper Ontology".to_owned()
            ]
        );
        assert_eq!(
            field_manifest.keywords_comma_separated(),
            "Communication,Field 33 Package,Organization,Upper Ontology"
        );
        assert_eq!(
            field_manifest.short_description(),
            "A short description of the field"
        );
        assert_eq!(
            field_manifest.description(),
            "A long description of the field."
        );
        assert_eq!(field_manifest.title(), "My interesting field");

        let literals = field_manifest
            .dependencies()
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<String>>();
        assert_eq!(
            literals,
            vec![
                "@fld33/communication ^0.1.0".to_owned(),
                "@fld33/organization ^0.1.1".to_owned(),
            ]
        );
    }
}
