#![allow(dead_code)]

use anyhow::{anyhow, Result};
use camino::Utf8Path;
use colored::Colorize;
use harriet::{
    Literal, Object, ObjectList, ParseError, Statement, Triples, TurtleDocument, Verb, Whitespace,
    IRI,
};
use lazy_static::lazy_static;
use plow_package_management::{
    package::FieldMetadata,
    registry::index::{IndexedPackageDependency, IndexedPackageVersion},
    resolve::Dependency,
    version::SemanticVersion,
};
use regex::Regex;
use serde_json::map;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

lazy_static! {
    static ref PACKAGE_FULL_NAME_REGEX: Regex = Regex::new(r#""(@.+/.+)""#).unwrap();
}

pub struct FieldManifest<'manifest> {
    extracted_annotations: HashMap<String, Result<Vec<String>, anyhow::Error>>,
    field_contents: String,
    pub ontology_iri: Option<String>,
    statements: Vec<Statement<'manifest>>,
}

impl<'manifest> std::fmt::Debug for FieldManifest<'manifest> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldManifest")
            .field("extracted_annotations", &self.extracted_annotations)
            .field("ontology_iri", &self.ontology_iri)
            .finish()
    }
}

impl<'manifest> FieldManifest<'manifest> {
    pub fn quick_extract_field_full_name<P: AsRef<Utf8Path>>(field_path: &P) -> Result<String> {
        let lines = crate::utils::read_lines(field_path.as_ref())?;
        let mut package_name_annotation_matched = false;
        #[allow(clippy::manual_flatten)]
        for line in lines {
            if let Ok(line) = line {
                if package_name_annotation_matched {
                    let captures = PACKAGE_FULL_NAME_REGEX.captures(&line);
                    if let Some(captures) = captures {
                        if let Some(package_full_name) = captures.get(1) {
                            return Ok(package_full_name.as_str().to_owned());
                        }
                    }
                } else if line.matches("registry:packageName").next().is_some() {
                    package_name_annotation_matched = true;
                    let captures = PACKAGE_FULL_NAME_REGEX.captures(&line);
                    if let Some(captures) = captures {
                        if let Some(package_full_name) = captures.get(1) {
                            return Ok(package_full_name.as_str().to_owned());
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        Err(anyhow!("Could not find package full name in field file"))
    }

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

        Ok(Self {
            extracted_annotations: prefixed_name_to_values_in_ttl,
            field_contents: field_contents.to_owned(),
            ontology_iri,
            statements,
        })
    }

    #[allow(clippy::unwrap_used)]
    #[allow(clippy::missing_panics_doc)]
    /// Assumes linted input
    pub fn make_field_metadata_from_manifest_unchecked(&self) -> FieldMetadata {
        let namespace = self.field_namespace().unwrap();
        let name = self.field_name().unwrap();
        let version = self.field_version().unwrap();
        #[allow(clippy::if_then_some_else_none)]
        let dependency_literals = self.field_dependency_literals();
        let dependencies =
            dependency_literals.map_or_else(std::vec::Vec::new, |dependency_literals| {
                dependency_literals
                    .iter()
                    .map(|literal| {
                        Dependency::<SemanticVersion>::try_from(literal.as_str()).unwrap()
                    })
                    .collect()
            });

        FieldMetadata {
            namespace,
            name,
            version,
            dependencies,
        }
    }

    pub fn make_index_from_manifest(&self) -> Result<IndexedPackageVersion> {
        let name = self.field_namespace_and_name().ok_or_else(|| {
            anyhow!("registry:packageName could not be found or malformed in manifest.")
        })?;

        let version = self.field_version().ok_or_else(|| {
            anyhow!("registry:packageVersion could not be found or malformed in manifest.")
        })?;

        #[allow(clippy::if_then_some_else_none)]
        let dependency_literals = self.field_dependency_literals();

        let deps = dependency_literals.map_or_else(std::vec::Vec::new, |dependency_literals| {
            dependency_literals
                .iter()
                .map(|literal| {
                    let literal_name_and_req = literal.split(' ').collect::<Vec<&str>>();
                    #[allow(clippy::indexing_slicing)]
                    IndexedPackageDependency {
                        name: literal_name_and_req[0].to_owned(),
                        req: literal_name_and_req[1].to_owned(),
                    }
                })
                .collect::<Vec<IndexedPackageDependency>>()
        });

        let mut hasher = Sha256::new();
        hasher.update(self.field_contents.as_bytes());
        let cksum = format!("{:X}", hasher.finalize()).to_lowercase();

        Ok(IndexedPackageVersion {
            name,
            version,
            cksum,
            ontology_iri: self.ontology_iri.clone(),
            deps,
        })
    }

    pub fn get_field_hash(&self) -> Result<String> {
        let namespace = self.field_namespace().ok_or_else(|| {
            anyhow!("registry:packageName could not be found or malformed in manifest.")
        })?;
        let name = self.field_name().ok_or_else(|| {
            anyhow!("registry:packageName could not be found or malformed in manifest.")
        })?;
        let version = self.field_version().ok_or_else(|| {
            anyhow!("registry:packageVersion could not be found or malformed in manifest.")
        })?;
        let field_hash = hash_field(namespace.as_ref(), name.as_ref(), version.as_ref());
        Ok(field_hash)
    }

    pub fn field_authors(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:author") {
            return Some(literals.clone());
        }
        None
    }

    // TODO: This can be probably done better.
    pub fn field_author_names(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:author") {
            let names = literals
                .iter()
                .map(|author| author.split('<').next())
                .filter_map(|name| {
                    if let Some(name) = name {
                        return Some(name.to_owned().trim().to_owned());
                    }
                    None
                })
                .collect::<Vec<String>>();
            return Some(names);
        }
        None
    }

    pub fn field_author_names_comma_separated(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:author") {
            let names = literals
                .iter()
                .map(|author| author.split('<').next())
                .filter_map(|name| {
                    if let Some(name) = name {
                        return Some(name.to_owned().trim().to_owned());
                    }
                    None
                })
                .collect::<Vec<String>>();
            let mut comma_separated_names = names.iter().fold("".to_owned(), |mut acc, name| {
                acc.push_str(name);
                acc.push(',');
                acc
            });
            comma_separated_names.pop();
            return Some(comma_separated_names);
        }
        None
    }

    // TODO: This can be probably done better.
    pub fn field_author_emails(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:author") {
            let emails = literals
                .iter()
                .map(|author| author.split('<').rev().next())
                .filter_map(|email| {
                    if let Some(email) = email {
                        if email.is_empty() {
                            return None;
                        }
                        #[allow(clippy::string_slice)]
                        #[allow(clippy::indexing_slicing)]
                        return Some(email.to_owned()[..email.len() - 1].to_owned());
                    }
                    None
                })
                .collect::<Vec<String>>();

            return Some(emails);
        }
        None
    }

    pub fn field_namespace_and_name(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:packageName") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_namespace(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:packageName") {
            if let Some(value) = literals.first() {
                return value.split('/').map(std::borrow::ToOwned::to_owned).next();
            }
            return None;
        }
        None
    }
    pub fn field_name(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:packageName") {
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

    pub fn field_categories(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:category") {
            return Some(literals.clone());
        }
        None
    }

    pub fn field_keywords(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:keyword") {
            return Some(literals.clone());
        }
        None
    }

    pub fn field_keywords_comma_separated(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:keyword") {
            let mut comma_separated_keywords =
                literals.iter().fold("".to_owned(), |mut acc, kw| {
                    acc.push_str(kw);
                    acc.push(',');
                    acc
                });
            comma_separated_keywords.pop();
            return Some(comma_separated_keywords);
        }
        None
    }

    pub fn field_version(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:packageVersion") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_repository_url(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:repository") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_homepage_url(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:homepage") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_documentation_url(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:documentation") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_license(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:license") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_license_spdx_literal(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:licenseSPDX") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }

    pub fn field_short_description(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:shortDescription") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }
    pub fn field_long_description(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("rdfs:comment") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }
    pub fn field_title(&self) -> Option<String> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("rdfs:label") {
            if let Some(value) = literals.first() {
                return Some(value.clone());
            }
            return None;
        }
        None
    }
    pub fn field_dependency_literals(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:dependency") {
            return Some(literals.clone());
        }
        None
    }

    pub fn update_owl_imports_and_serialize(
        &self,
        new_object_list: ObjectList<'manifest>,
        mut statements: Vec<Statement<'manifest>>,
        target_path: &Utf8Path,
        remove_owl_imports_line: bool,
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

        let mut serialized_turtle = new_doc.to_string();

        // Hacky way to remove owl imports, I'm deferring the refactoring of this to the refactoring effort PRs
        // This assumes that the left over part in the file is ..;\nowl:imports..
        if remove_owl_imports_line {
            let (rest, _) = serialized_turtle.split_once("owl:imports").unwrap();
            serialized_turtle = rest.to_owned();
            serialized_turtle = serialized_turtle[..serialized_turtle.len() - 2].to_owned();
            serialized_turtle += " .";
        }

        std::fs::write(target_path, serialized_turtle).unwrap();
    }

    pub fn create_owl_imports_and_serialize(
        &self,
        new_predicate: (
            Option<Whitespace<'manifest>>,
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

    #[allow(clippy::restriction)]
    #[allow(clippy::missing_panics_doc)]
    pub fn field_dependency_names(&self) -> Option<Vec<String>> {
        if let Some(Ok(literals)) = self.extracted_annotations.get("registry:dependency") {
            let names = literals
                .iter()
                .map(|l| l.split(' ').next())
                .collect::<Vec<Option<&str>>>();
            if names.iter().any(std::option::Option::is_none) {
                return None;
            }
            let names: Vec<String> = names.iter().map(|n| n.unwrap().to_owned()).collect();
            return Some(names);
        }
        None
    }
}

/// Generates a sha256 hash from field name, namespace and version.
fn hash_field(namespace: &str, name: &str, version: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}/{} {}", namespace, name, version));
    format!("{:x}", hasher.finalize())
}

fn get_string_literal_from_object(object: &Object) -> anyhow::Result<String> {
    match object {
        Object::Literal(literal) => match literal {
            Literal::RDFLiteral(rdf_literal) => {
                let turtle_string = &rdf_literal.string;
                Ok(turtle_string.to_string())
            }
            // TODO: Implement when needed..
            Literal::BooleanLiteral(_) => anyhow::bail!("Boolean literal found in RDF literal"),
            Literal::NumericLiteral(_) => anyhow::bail!("Numeric literal found in RDF literal"),
        },

        _ => anyhow::bail!("Boolean literal found in RDF literal"),
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
        assert_eq!(field_manifest.field_name(), Some("test".to_owned()));
        assert_eq!(field_manifest.field_namespace(), Some("@fld33".to_owned()));
        assert_eq!(
            field_manifest.field_namespace_and_name(),
            Some("@fld33/test".to_owned())
        );
        assert_eq!(field_manifest.field_version(), Some("0.1.2".to_owned()));
        assert_eq!(
            field_manifest.field_authors(),
            Some(vec![
                "Miles Davis <miles@field33.com>".to_owned(),
                "Joe Pass <joe@field33.com>".to_owned()
            ])
        );
        assert_eq!(
            field_manifest.field_author_names(),
            Some(vec!["Miles Davis".to_owned(), "Joe Pass".to_owned()])
        );
        assert_eq!(
            field_manifest.field_author_names_comma_separated(),
            Some("Miles Davis,Joe Pass".to_owned())
        );
        assert_eq!(
            field_manifest.field_author_emails(),
            Some(vec![
                "miles@field33.com".to_owned(),
                "joe@field33.com".to_owned()
            ])
        );
        assert_eq!(field_manifest.field_homepage_url(), None);
        assert_eq!(
            field_manifest.field_repository_url(),
            Some(
                "https://github.com/field33/ontology-workspace/tree/main/%40fld33/test".to_owned()
            )
        );
        assert_eq!(
            field_manifest.field_documentation_url(),
            Some("https://field33.com".to_owned())
        );
        assert_eq!(
            field_manifest.field_license(),
            Some("MIT License".to_owned())
        );
        assert_eq!(
            field_manifest.field_license_spdx_literal(),
            Some("MIT".to_owned())
        );
        assert_eq!(
            field_manifest.field_categories(),
            Some(vec![
                "Communication".to_owned(),
                "Organization".to_owned(),
                "Upper Ontology".to_owned()
            ])
        );
        assert_eq!(
            field_manifest.field_keywords(),
            Some(vec![
                "Communication".to_owned(),
                "Field 33 Package".to_owned(),
                "Organization".to_owned(),
                "Upper Ontology".to_owned()
            ])
        );
        assert_eq!(
            field_manifest.field_keywords_comma_separated(),
            Some("Communication,Field 33 Package,Organization,Upper Ontology".to_owned())
        );
        assert_eq!(
            field_manifest.field_short_description(),
            Some("A short description of the field".to_owned())
        );
        assert_eq!(
            field_manifest.field_long_description(),
            Some("A long description of the field.".to_owned())
        );
        assert_eq!(
            field_manifest.field_title(),
            Some("My interesting field".to_owned())
        );
        assert_eq!(
            field_manifest.field_dependency_literals(),
            Some(vec![
                "@fld33/communication ^0.1.0".to_owned(),
                "@fld33/organization ^0.1.1".to_owned(),
            ])
        );
    }
}
