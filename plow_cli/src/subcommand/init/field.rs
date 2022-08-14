use crate::error::CliError;
use crate::error::FieldInitializationError::*;
use harriet::Statement;
use harriet::Whitespace;
use harriet::{
    BaseDirective, IRIReference, Object, ObjectList, PredicateObjectList, PrefixDirective,
    PrefixedName, Subject, Triples, TurtleDocument, IRI,
};
use harriet::{Directive, Literal, RDFLiteral, StringLiteralQuote, TurtleString};
use std::borrow::Cow;

const REGISTRY_PREFIX: &str = "registry";
const REGISTRY_PREFIX_IRI: &str = "http://field33.com/ontologies/REGISTRY/";

#[derive(Debug, Default)]
pub struct FieldName(String);

#[allow(clippy::indexing_slicing)]
impl FieldName {
    /// Returns the namespace of the package name (including `@` prefix).
    pub fn namespace(&self) -> &str {
        let parts = self.0.split('/').collect::<Vec<_>>();
        parts[0]
    }

    /// Returns the name of the package.
    pub fn name(&self) -> &str {
        let parts = self.0.split('/').collect::<Vec<_>>();
        parts[1]
    }
}

impl ToString for FieldName {
    fn to_string(&self) -> String {
        format!(
            "{namespace}/{name}",
            namespace = self.namespace(),
            name = self.name()
        )
    }
}

#[allow(clippy::indexing_slicing)]
impl TryFrom<String> for FieldName {
    type Error = CliError;
    fn try_from(package_name: String) -> Result<Self, Self::Error> {
        let parts = package_name.split('/').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(InvalidFieldNameProvided {
                reason: "An example of a valid name @namespace/name".to_owned(),
            }
            .into());
        }

        let is_alphanumeric_namespace = parts[0]
            .chars()
            .all(|c| char::is_alphanumeric(c) || c == '_' || c == '@');

        let is_alphanumeric_name = parts[1]
            .chars()
            .all(|c| char::is_alphanumeric(c) || c == '_');

        if !is_alphanumeric_namespace
            || !is_alphanumeric_name
            || !parts[0].starts_with('@')
            || parts[0].matches('@').count() > 1
        {
            return Err(InvalidFieldNameProvided {
                reason: "Name may only contain alphanumeric characters and underscores. An example of a valid name @namespace/name".to_owned(),
            }
            .into());
        }
        let contains_double_underscores = package_name.contains("__");
        if contains_double_underscores {
            return Err(InvalidFieldNameProvided {
                reason: "Name may not contain two underscores after each other `__`. An example of a valid name @namespace/name".to_owned(),
            }
            .into());
        }
        Ok(Self(package_name))
    }
}

/// Creates a new field that is conforming to the FLD33 field format.
pub fn new(name: &str) -> Result<String, CliError> {
    let field_full_name = FieldName::try_from(name.to_owned())?;
    let field_iri = format!("http://field33.com/ontologies/{name}/", name = name);
    let mut field = TurtleDocument {
        statements: vec![],
        trailing_whitespace: None,
    };

    // Directives
    let mut prefix_directives = vec![PrefixDirective {
        leading_whitespace: None,
        prefix: None,
        iri: IRIReference {
            iri: Cow::Borrowed(&field_iri),
        },
    }];
    prefix_directives.append(&mut default_prefix_directives());

    for directive in prefix_directives {
        field
            .statements
            .push(Statement::Directive(Directive::Prefix(directive)));
    }

    field
        .statements
        .push(Statement::Directive(Directive::Base(BaseDirective {
            leading_whitespace: None,
            iri: IRIReference {
                iri: Cow::Borrowed(&field_iri),
            },
        })));

    // Triple for field declaration
    field.statements.push(Statement::Triples(Triples::Labeled(
        None,
        Subject::IRI(IRI::IRIReference(IRIReference {
            iri: Cow::Borrowed(&field_iri),
        })),
        PredicateObjectList {
            list: vec![
                (
                    Whitespace {
                        whitespace: "".into(),
                    },
                    harriet::Verb::IRI(IRI::PrefixedName(PrefixedName {
                        prefix: Some(Cow::Borrowed("rdf")),
                        name: Some(Cow::Borrowed("type")),
                    })),
                    ObjectList {
                        list: vec![(
                            None,
                            None,
                            Object::IRI(IRI::PrefixedName(PrefixedName {
                                prefix: Some(Cow::Borrowed("owl")),
                                name: Some(Cow::Borrowed("field")),
                            })),
                        )],
                    },
                    None,
                ),
                make_predicate_stringy_object("registry", "fieldFormatVersion", "v1"),
                make_predicate_stringy_object("registry", "packageName", name),
                make_predicate_stringy_object("registry", "packageVersion", "0.1.0"),
                make_predicate_stringy_object(
                    "registry",
                    "canonicalPrefix",
                    field_full_name.name(),
                ),
            ],
        },
    )));

    Ok(field.to_string())
}

fn default_prefix_directives<'directive>() -> Vec<PrefixDirective<'directive>> {
    vec![
        make_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
        make_prefix("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
        make_prefix("xml", "http://www.w3.org/XML/1998/namespace"),
        make_prefix("xsd", "http://www.w3.org/2001/XMLSchema#"),
        make_prefix("owl", "http://www.w3.org/2002/07/owl#"),
        make_prefix("owl", "http://www.w3.org/2002/07/owl#"),
        make_prefix(REGISTRY_PREFIX, REGISTRY_PREFIX_IRI),
    ]
}

const fn make_prefix<'directive>(
    prefix: &'directive str,
    iri: &'directive str,
) -> PrefixDirective<'directive> {
    PrefixDirective {
        prefix: Some(Cow::Borrowed(prefix)),
        iri: IRIReference {
            iri: Cow::Borrowed(iri),
        },
        leading_whitespace: None,
    }
}

fn make_predicate_stringy_object<'list>(
    predicate_prefix: &'list str,
    predicate_name: &'list str,
    object_literal: &'list str,
) -> (
    Whitespace<'list>,
    harriet::Verb<'list>,
    ObjectList<'list>,
    Option<Whitespace<'list>>,
) {
    (
        Whitespace {
            whitespace: "".into(),
        },
        harriet::Verb::IRI(IRI::PrefixedName(PrefixedName {
            prefix: Some(Cow::Borrowed(predicate_prefix)),
            name: Some(Cow::Borrowed(predicate_name)),
        })),
        ObjectList {
            list: vec![(
                None,
                None,
                Object::Literal(Literal::RDFLiteral(RDFLiteral {
                    string: TurtleString::StringLiteralQuote(StringLiteralQuote {
                        string: Cow::Borrowed(object_literal),
                    }),
                    language_tag: None,
                    iri: None,
                })),
            )],
        },
        None,
    )
}
