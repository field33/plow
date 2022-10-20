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
#[allow(clippy::too_many_lines)]
pub fn new(name: &str) -> std::string::String {
    let field_iri = format!("http://field33.com/ontologies/{name}/", name = name);
    let mut field = TurtleDocument {
        statements: vec![],
        trailing_whitespace: None,
    };

    field
        .statements
        .push(Statement::Directive(Directive::Base(BaseDirective {
            leading_whitespace: Some(Whitespace {
                whitespace: "\n".into(),
            }),
            iri: IRIReference {
                iri: Cow::Borrowed(&field_iri),
            },
        })));

    // Directives
    let mut prefix_directives = vec![PrefixDirective {
        leading_whitespace: Some(Whitespace {
            whitespace: "\n".into(),
        }),
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

    // Triple for field declaration
    field.statements.push(Statement::Triples(Triples::Labeled(
        Some(Whitespace {
            whitespace: "\n\n".into(),
        }),
        Subject::IRI(IRI::IRIReference(IRIReference {
            iri: Cow::Borrowed(&field_iri),
        })),
        PredicateObjectList {
            list: vec![
                (
                    Whitespace {
                        whitespace: " ".into(),
                    },
                    harriet::Verb::IRI(IRI::PrefixedName(PrefixedName {
                        prefix: Some(Cow::Borrowed("rdf")),
                        name: Some(Cow::Borrowed("type")),
                    })),
                    ObjectList {
                        list: vec![(
                            None,
                            Some(Whitespace {
                                whitespace: " ".into(),
                            }),
                            Object::IRI(IRI::PrefixedName(PrefixedName {
                                prefix: Some(Cow::Borrowed("owl")),
                                name: Some(Cow::Borrowed("Ontology")),
                            })),
                        )],
                    },
                    Some(Whitespace {
                        whitespace: " ".into(),
                    }),
                ),
           
                make_predicate_stringy_object("registry", "author", "John Doe <john@example.com>", Some(
                    "\n\n# Specifies the field author in a format as in the example.\n#"
                    ),None),
                make_predicate_stringy_object("registry", "packageName", name, Some(
                    "\n\n# Name of your field in the form of @namespace/name.\n"
                    ),None),
                make_predicate_stringy_object("registry", "packageVersion", "0.1.0", Some(
                    "\n\n# A bare semantic version for your field.\n"
                    ),None),
                make_predicate_stringy_object(
                    "registry",
                    "category",
                    "Communication\", \"Core\", \"Design", Some(
                    "\n\n# You may specify a maximum of 5 categories to categorize your field as in the example. Available categories could be viewed in <https://registry.field33.com>\n#"
                    ),None,
                ),
                make_predicate_stringy_object(
                    "registry",
                    "keyword",
                    "some\", \"key\", \"words",Some(
                    "\n\n# You may specify a maximum of 5 keywords to describe your field as in the example.\n#"
                    ),
                    None,
                ),
                make_predicate_stringy_object(
                    "registry",
                    "shortDescription",
                    "A short description for the field",Some(
                    "\n\n# Specify a short description for your field to be viewed in registry.field33.com. The value requires a language tag \"My short description.\"@en\n#"
                    ),
                    Some(
                        "en"
                    ),
                ),
                make_predicate_stringy_object("rdfs", "comment", "A description for the field",  Some(
                    "\n\n# Specify a description for your field to be viewed in registry.field33.com. The value requires a language tag \"My Description\"@en\n#"
                    ),  Some(
                        "en"
                    )),
                make_predicate_stringy_object("rdfs", "label", "A title for the field", Some(
                    "\n\n# Specify a title for your field to be viewed in registry.field33.com. The value requires a language tag \"My title\"@en\n#"
                    ),
                      Some(
                        "en"
                    )
                ),

                make_predicate_stringy_object("registry", "dependency", "@namespace/name <version requirement>\", \"@namespace/name <version requirement>\" and so on..", Some(
                    "\n\n# If the field has dependencies you may comment the following section out and specify them as in the example.\n# "
                ),None),
                make_predicate_stringy_object("registry", "repository", "<a valid domain name>", Some(
                    "\n\n# Specifying a repository url could be helpful to lead the user to your workspace in github, gitlab etc.\n# "
                ),None),
                make_predicate_stringy_object("registry", "homepage", "<a valid domain name>", Some(
                    "\n\n# Specifying a homepage url could be helpful to give more info about your project to the user.\n# "
                ),None),
                make_predicate_stringy_object("registry", "documentation", "<a valid domain name>", Some(
                    "\n\n# Specifying a documentation url could be helpful to for the user to learn more about your field.\n# "
                ),None),
                make_predicate_stringy_object("registry", "license", "<a license description>", Some(
                    "\n\n# You may specify an additional license description here. \n# "
                ),None),
                make_predicate_stringy_object("registry", "licenseSPDX", "MIT", Some(
                    "\n\n# Specify a valid SPDX license for your field, valid licenses could be viewed in <https://spdx.org/licenses>.\n"
                    ), None),

                make_predicate_stringy_object("registry", "ontologyFormatVersion", "v1", Some("\n\n"), None),

            ],
        },
    )));

    field.to_string()
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

fn make_prefix<'directive>(
    prefix: &'directive str,
    iri: &'directive str,
) -> PrefixDirective<'directive> {
    PrefixDirective {
        prefix: Some(Cow::Borrowed(prefix)),
        iri: IRIReference {
            iri: Cow::Borrowed(iri),
        },
        leading_whitespace: Some(Whitespace {
            whitespace: "\n".into(),
        }),
    }
}

pub fn make_predicate_stringy_object<'list>(
    predicate_prefix: &'list str,
    predicate_name: &'list str,
    object_literal: &'list str,
    comment_out_with_explanation: Option<&'list str>,
    language_tag: Option<&'list str>,
) -> (
    Whitespace<'list>,
    harriet::Verb<'list>,
    ObjectList<'list>,
    Option<Whitespace<'list>>,
) {
    (
        comment_out_with_explanation.map_or_else(
            || Whitespace {
                whitespace: "\n".into(),
            },
            |comment_out_with_explanation| Whitespace {
                whitespace: comment_out_with_explanation.into(),
            },
        ),
        harriet::Verb::IRI(IRI::PrefixedName(PrefixedName {
            prefix: Some(Cow::Borrowed(predicate_prefix)),
            name: Some(Cow::Borrowed(predicate_name)),
        })),
        ObjectList {
            list: vec![(
                None,
                Some(Whitespace {
                    whitespace: " ".into(),
                }),
                Object::Literal(Literal::RDFLiteral(RDFLiteral {
                    string: TurtleString::StringLiteralQuote(StringLiteralQuote {
                        string: Cow::Borrowed(object_literal),
                    }),
                    language_tag: language_tag.map(std::convert::Into::into),
                    iri: None,
                })),
            )],
        },
        Some(Whitespace {
            whitespace: " ".into(),
        }),
    )
}

pub fn make_predicate_object<'list>(
    predicate_prefix: &'list str,
    predicate_name: &'list str,
    object_list: ObjectList<'list>,
) -> (
    Whitespace<'list>,
    harriet::Verb<'list>,
    ObjectList<'list>,
    Option<Whitespace<'list>>,
) {
    (
        Whitespace {
            whitespace: "\n".into(),
        },
        harriet::Verb::IRI(IRI::PrefixedName(PrefixedName {
            prefix: Some(Cow::Borrowed(predicate_prefix)),
            name: Some(Cow::Borrowed(predicate_name)),
        })),
        object_list,
        Some(Whitespace {
            whitespace: " ".into(),
        }),
    )
}
