#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    // Group of too restrictive lints
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::enum_glob_use,
    clippy::wildcard_enum_match_arm,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::multiple_crate_versions,
    clippy::default_numeric_fallback,
    clippy::map_err_ignore,

    // We decided that we're ok with expect
    clippy::expect_used,

    // Too restrictive for the current style
    clippy::missing_inline_in_public_items,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,
    // Currently breaks CI, let's wait a bit more until new clippy version is more spread.
    // clippy::single_char_lifetime_names,

    // Allowed lints related to cargo
    // (comment these out if you'd like to improve Cargo.toml)
    clippy::wildcard_dependencies,
    clippy::redundant_feature_names,
    clippy::cargo_common_metadata,

    // Comment these out when writing docs
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,

    // Comment these out before submitting a PR
    clippy::todo,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,

    clippy::negative_feature_names
)]

pub mod constants;

use crate::constants::{REGISTRY_PREFIX, REGISTRY_PREFIX_IRI};
use anyhow::{anyhow, bail};
use harriet::Statement;
use harriet::{
    BaseDirective, IRIReference, Item, Object, ObjectList, PredicateObjectList, PrefixDirective,
    PrefixedName, Subject, Triples, TurtleDocument, IRI,
};
use harriet::{Directive, Literal, RDFLiteral, StringLiteralQuote, TurtleString};
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct PackageName(String);

impl ToString for PackageName {
    fn to_string(&self) -> String {
        format!(
            "{namespace}/{name}",
            namespace = self.namespace(),
            name = self.package()
        )
    }
}

#[allow(clippy::indexing_slicing)]
impl PackageName {
    /// Returns the namespace of the package name (including `@` prefix).
    pub fn namespace(&self) -> &str {
        let parts = self.0.split('/').collect::<Vec<_>>();
        parts[0]
    }

    /// Returns the name of the package.
    pub fn package(&self) -> &str {
        let parts = self.0.split('/').collect::<Vec<_>>();
        parts[1]
    }
}

#[allow(clippy::indexing_slicing)]
impl TryFrom<String> for PackageName {
    type Error = anyhow::Error;
    fn try_from(package_name: String) -> Result<Self, Self::Error> {
        // TODO: verify allowed characters
        let parts = package_name.split('/').collect::<Vec<_>>();
        if parts.len() != 2 {
            bail!("Invalid package name `{package_name}` - Package name must consist of namespace and per-namespace-name", package_name = package_name );
        }
        if parts[0]
            .chars()
            .next()
            .ok_or_else(|| anyhow!("Package name prefix doesn't consist of any characters"))?
            != '@'
        {
            bail!("Invalid namespace `{namespace}` of `{package_name}` - Namespace must have `@` as a first character", package_name = package_name, namespace= parts[0] );
        }
        Ok(Self(package_name))
    }
}

/// Creates a new ontology that is conforming to the FLD33 ontology format.
pub fn initialize_ontology(name: &str) -> Result<String, anyhow::Error> {
    let package_name = PackageName::try_from(name.to_owned())?;

    let ontology_iri = format!("http://field33.com/ontologies/{name}/", name = name);

    let mut ontology = TurtleDocument { items: vec![] };

    // directives
    let mut prefix_directives = vec![PrefixDirective {
        prefix: None,
        iri: IRIReference {
            iri: Cow::Borrowed(&ontology_iri),
        },
    }];
    prefix_directives.append(&mut default_prefix_directives());

    for directive in prefix_directives {
        ontology
            .items
            .push(Item::Statement(Statement::Directive(Directive::Prefix(
                directive,
            ))));
    }

    ontology
        .items
        .push(Item::Statement(Statement::Directive(Directive::Base(
            BaseDirective {
                iri: IRIReference {
                    iri: Cow::Borrowed(&ontology_iri),
                },
            },
        ))));
    // Triple for ontology declaration
    ontology
        .items
        .push(Item::Statement(Statement::Triples(Triples::Labeled(
            Subject::IRI(IRI::IRIReference(IRIReference {
                iri: Cow::Borrowed(&ontology_iri),
            })),
            PredicateObjectList {
                list: vec![
                    (
                        IRI::PrefixedName(PrefixedName {
                            prefix: Some(Cow::Borrowed("rdf")),
                            name: Some(Cow::Borrowed("type")),
                        }),
                        ObjectList {
                            list: vec![Object::IRI(IRI::PrefixedName(PrefixedName {
                                prefix: Some(Cow::Borrowed("owl")),
                                name: Some(Cow::Borrowed("Ontology")),
                            }))],
                        },
                    ),
                    make_predicate_stringy_object("registry", "ontologyFormatVersion", "v1"),
                    make_predicate_stringy_object("registry", "packageName", name),
                    make_predicate_stringy_object("registry", "packageVersion", "0.1.0"),
                    make_predicate_stringy_object(
                        "registry",
                        "canonicalPrefix",
                        package_name.package(),
                    ),
                ],
            },
        ))));

    Ok(ontology.to_string())
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
    }
}

fn make_predicate_stringy_object<'list>(
    predicate_prefix: &'list str,
    predicate_name: &'list str,
    object_literal: &'list str,
) -> (IRI<'list>, ObjectList<'list>) {
    (
        IRI::PrefixedName(PrefixedName {
            prefix: Some(Cow::Borrowed(predicate_prefix)),
            name: Some(Cow::Borrowed(predicate_name)),
        }),
        ObjectList {
            list: vec![Object::Literal(Literal::RDFLiteral(RDFLiteral {
                string: TurtleString::StringLiteralQuote(StringLiteralQuote {
                    string: Cow::Borrowed(object_literal),
                }),
                language_tag: None,
                iri: None,
            }))],
        },
    )
}

pub fn validate_ontology_name(input: &str) -> Result<(), &'static str> {
    let is_alphanumeric = input.chars().all(|c| char::is_alphanumeric(c) || c == '_');
    if !is_alphanumeric {
        return Err("Name may only contain alphanumeric characters and underscores");
    }
    let contains_double_underscores = input.contains("__");
    if contains_double_underscores {
        return Err("Name may not contain two underscores after each other `__`");
    }

    Ok(())
}
