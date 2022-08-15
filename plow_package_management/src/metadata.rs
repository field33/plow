//! Provides a metadata view on an ontology file (that has previously been validated).

use anyhow::{anyhow, bail, Context};
use harriet::{Directive, Statement, TurtleDocument};
use plow_graphify::document_to_graph;
use plow_ontology::constants::{
    REGISTRY_CANONICAL_PREFIX, REGISTRY_DEPENDENCY, REGISTRY_ONTOLOGY_FORMAT_VERSION,
    REGISTRY_PACKAGE_NAME, REGISTRY_PACKAGE_VERSION,
};
use rdftk_core::model::graph::GraphRef;
use rdftk_iri::IRI as RDFTK_IRI;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;
use std::str::FromStr;

use crate::package::PackageVersion;
use crate::resolve::Dependency;
use crate::version::SemanticVersion;

#[derive(Debug, Clone)]
pub enum OntologyFormatVersion {
    V1,
}

impl FromStr for OntologyFormatVersion {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "v1" => Ok(Self::V1),
            _ => Err(anyhow!(
                "Unrecognized ontology format version: {input}",
                input = input
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OntologyMetadata {
    #[serde(skip)]
    pub ontology_format_version: OntologyFormatVersion,
    pub root_prefix: String,
    pub canonical_prefix: String,
    pub dependencies: Vec<Dependency<SemanticVersion>>,
    pub package_name: String,
    pub package_version: SemanticVersion,
}

impl OntologyMetadata {
    fn get_stringy_ontology_annotation(
        rdf_graph: &GraphRef,
        root_prefix: &str,
        annotation_property_iri: &str,
    ) -> Result<String, anyhow::Error> {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        let rdf_graph_borrow = rdf_graph.borrow();

        // We explicitly pass valid data, unwrap is safe here.
        #[allow(clippy::unwrap_used)]
        let annotations = rdf_graph_borrow
            .statements()
            .filter(|statement| {
                statement.subject()
                    == &rdf_factory.named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                    && statement.predicate()
                        == &RDFTK_IRI::from_str(annotation_property_iri).unwrap().into()
            })
            .collect::<HashSet<_>>();

        let annotation = annotations.iter().next().ok_or_else(|| {
            anyhow!(
                "No annotation found for annotation property: `{}`",
                annotation_property_iri
            )
        })?;
        let literal = annotation
            .object()
            .as_literal()
            .ok_or_else(|| anyhow!("annotation value is not a literal"))?;

        Ok(literal.lexical_form().as_str().to_owned())
    }

    fn get_dependency_strings(
        rdf_graph: &GraphRef,
        root_prefix: &str,
    ) -> Result<Vec<String>, anyhow::Error> {
        let rdf_factory = rdftk_core::simple::statement::statement_factory();
        let rdf_graph_borrow = rdf_graph.borrow();

        // We explicitly pass valid data, unwrap is safe here.
        #[allow(clippy::unwrap_used)]
        let annotations = rdf_graph_borrow
            .statements()
            .filter(|statement| {
                statement.subject()
                    == &rdf_factory.named_subject(RDFTK_IRI::from_str(root_prefix).unwrap().into())
                    && statement.predicate()
                        == &RDFTK_IRI::from_str(REGISTRY_DEPENDENCY).unwrap().into()
            })
            .collect::<HashSet<_>>();

        let dependency_literals = annotations
            .into_iter()
            .map(|annotation| {
                let literal = annotation
                    .object()
                    .as_literal()
                    .ok_or_else(|| anyhow!("annotation value is not a literal"))?;

                Ok(literal.lexical_form().as_str().to_owned())
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?;

        Ok(dependency_literals)
    }

    fn get_ontology_format_version(
        rdf_graph: &GraphRef,
        root_prefix: &str,
    ) -> Result<OntologyFormatVersion, anyhow::Error> {
        let literal_value = Self::get_stringy_ontology_annotation(
            rdf_graph,
            root_prefix,
            REGISTRY_ONTOLOGY_FORMAT_VERSION,
        )?;
        OntologyFormatVersion::from_str(&literal_value)
    }

    fn get_canonical_prefix(
        rdf_graph: &GraphRef,
        root_prefix: &str,
    ) -> Result<String, anyhow::Error> {
        let literal_value = Self::get_stringy_ontology_annotation(
            rdf_graph,
            root_prefix,
            REGISTRY_CANONICAL_PREFIX,
        )?;
        Ok(literal_value)
    }

    fn get_package_name(rdf_graph: &GraphRef, root_prefix: &str) -> Result<String, anyhow::Error> {
        let literal_value =
            Self::get_stringy_ontology_annotation(rdf_graph, root_prefix, REGISTRY_PACKAGE_NAME)?;
        Ok(literal_value)
    }

    fn get_package_version(
        rdf_graph: &GraphRef,
        root_prefix: &str,
    ) -> Result<SemanticVersion, anyhow::Error> {
        let literal_value = Self::get_stringy_ontology_annotation(
            rdf_graph,
            root_prefix,
            REGISTRY_PACKAGE_VERSION,
        )?;

        // We require that the version predicates which are fed to the resolver are either bare or exact but always complete.
        // This function ensures that this is the case.
        if let Ok(semver) = SemanticVersion::try_from(&literal_value) {
            let bare_and_complete = literal_value.replace('.', "").chars().all(char::is_numeric)
                && (literal_value.matches('.').count() == 2
                    && literal_value.split('.').count() == 3);

            if bare_and_complete {
                return Ok(semver);
            }
            bail!("Expected bare and complete version, got {literal_value}",);
        }
        bail!("Invalid version predicate {literal_value}",);
    }
}

pub fn get_root_prefix<'document>(
    document: &'document TurtleDocument,
) -> Option<&'document Cow<'document, str>> {
    let mut root_prefix_directive = None;
    for statement in &document.statements {
        if let Statement::Directive(Directive::Prefix(directive)) = statement {
            if directive.prefix.is_none() {
                root_prefix_directive = Some(directive);
            }
        }
    }

    root_prefix_directive.map(|n| &n.iri.iri)
}

impl TryFrom<&TurtleDocument<'_>> for OntologyMetadata {
    type Error = anyhow::Error;

    fn try_from<'document>(document: &TurtleDocument) -> Result<Self, Self::Error> {
        let rdf_graph = document_to_graph(document).context("Failed to parse turtle document")?;

        let root_prefix =
            get_root_prefix(document).ok_or_else(|| anyhow!("Unable to get root prefix"))?;

        let ontology_format_version = Self::get_ontology_format_version(&rdf_graph, root_prefix)?;

        let dependencies = Self::get_dependency_strings(&rdf_graph.clone(), root_prefix)?
            .iter()
            .map(|dep_string| Dependency::<SemanticVersion>::try_from(dep_string.as_str()))
            .collect::<Result<Vec<_>, anyhow::Error>>()?;

        Ok(Self {
            root_prefix: root_prefix.to_string(),
            ontology_format_version,
            canonical_prefix: Self::get_canonical_prefix(&rdf_graph, root_prefix)?,
            dependencies,
            package_name: Self::get_package_name(&rdf_graph, root_prefix)?,
            package_version: Self::get_package_version(&rdf_graph, root_prefix)?,
        })
    }
}

// Only one way conversion is allowed.
#[allow(clippy::from_over_into)]
impl Into<PackageVersion> for OntologyMetadata {
    fn into(self) -> PackageVersion {
        PackageVersion {
            package_name: self.package_name,
            version: self.package_version.to_string(),
        }
    }
}
