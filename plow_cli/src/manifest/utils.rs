use crate::registry::Dependency;
use crate::registry::SemanticVersion;
use anyhow::{anyhow, Result};
use camino::Utf8Path;
use harriet::{
    Literal, Object, ObjectList, ParseError, Statement, Triples, TurtleDocument, Verb, IRI,
};
use lazy_static::lazy_static;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;

lazy_static! {
    static ref PACKAGE_FULL_NAME_REGEX: Regex = Regex::new(r#""(@.+/.+)""#).unwrap();
}

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

pub fn get_string_literal_from_object(object: &Object) -> anyhow::Result<String> {
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
