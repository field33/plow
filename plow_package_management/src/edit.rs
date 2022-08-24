use anyhow::anyhow;
use harriet::{
    IRIReference, Literal, Object, ObjectList, PredicateObjectList, PrefixedName, RDFLiteral,
    Statement, StringLiteralQuote, Subject, Triples, TurtleDocument, TurtleString, Verb,
    Whitespace, IRI,
};
use plow_ontology::constants::REGISTRY_DEPENDENCY;
use std::borrow::Cow;

use crate::resolve::Dependency;
use crate::version::SemanticVersion;

pub trait EditOperation {
    fn apply(&self, document: &mut TurtleDocument) -> Result<(), anyhow::Error>;
}

pub struct AddDependency {
    /// IRI of the ontology to add the dependency to
    pub ontology_iri: String,
    pub dependency: Dependency<SemanticVersion>,
    pub dependency_ontology_iri: String,
}

impl EditOperation for AddDependency {
    fn apply(&self, document: &mut TurtleDocument) -> Result<(), anyhow::Error> {
        let annotation = Statement::Triples(Triples::Labeled(
            None,
            Subject::IRI(IRI::IRIReference(IRIReference {
                iri: Cow::from(self.ontology_iri.clone()),
            })),
            PredicateObjectList {
                list: vec![
                    (
                        Whitespace {
                            whitespace: "".into(),
                        },
                        harriet::Verb::IRI(IRI::IRIReference(IRIReference {
                            iri: Cow::Borrowed(REGISTRY_DEPENDENCY),
                        })),
                        ObjectList {
                            list: vec![(
                                None,
                                None,
                                Object::Literal(Literal::RDFLiteral(RDFLiteral {
                                    string: TurtleString::StringLiteralQuote(StringLiteralQuote {
                                        string: Cow::Owned(self.dependency.to_string()),
                                    }),
                                    language_tag: None,
                                    iri: None,
                                })),
                            )],
                        },
                        None,
                    ),
                    (
                        Whitespace {
                            whitespace: "".into(),
                        },
                        harriet::Verb::IRI(IRI::PrefixedName(PrefixedName {
                            prefix: Some(Cow::Borrowed("owl")),
                            name: Some(Cow::Borrowed("imports")),
                        })),
                        ObjectList {
                            list: vec![(
                                None,
                                None,
                                Object::IRI(IRI::IRIReference(IRIReference {
                                    iri: Cow::Owned(self.dependency_ontology_iri.clone()),
                                })),
                            )],
                        },
                        None,
                    ),
                ],
            },
        ));

        document.statements.push(annotation);
        Ok(())
    }
}

pub struct RemoveDependency {
    pub ontology_iri: String,
    pub dependency_name: String,
}

impl EditOperation for RemoveDependency {
    fn apply(&self, document: &mut TurtleDocument) -> Result<(), anyhow::Error> {
        let mut dependency_found = false;

        let ontology_iri_subject = Subject::IRI(IRI::IRIReference(IRIReference {
            iri: Cow::Owned(self.ontology_iri.clone()),
        }));

        let dependency_predicate = IRI::IRIReference(IRIReference {
            iri: Cow::Owned(REGISTRY_DEPENDENCY.to_owned()),
        });
        let dependency_predicate_prefixed = IRI::PrefixedName(PrefixedName {
            prefix: Some(Cow::from("registry")),
            name: Some(Cow::from("dependency")),
        });
        for statement in &mut document.statements {
            if let Statement::Triples(Triples::Labeled(_, subject, ref mut predicate_object_list)) =
                statement
            {
                if subject != &ontology_iri_subject {
                    continue;
                }
                'predicate_object_list: for (_, verb, object_list, _) in
                    &mut predicate_object_list.list
                {
                    match verb {
                        Verb::IRI(predicate) => {
                            if predicate != &dependency_predicate
                                && predicate != &dependency_predicate_prefixed
                            {
                                continue 'predicate_object_list;
                            }
                            let matching_dependency = object_list.list.iter().enumerate().find(
                                |(_index, (_, _, object))| {
                                    if let Object::Literal(Literal::RDFLiteral(RDFLiteral {
                                        string,
                                        ..
                                    })) = object
                                    {
                                        if let Ok(dep) = Dependency::<SemanticVersion>::try_from(
                                            string.to_string().as_str(),
                                        ) {
                                            dep.full_name == self.dependency_name
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                },
                            );
                            if let Some((dep_index, _)) = matching_dependency {
                                object_list.list.remove(dep_index);
                                dependency_found = true;
                            }
                        }
                        Verb::A => {
                            // Currently ignored
                        }
                    }
                }
                let mut now_empty_index: Option<usize> = None;
                predicate_object_list.list.iter().enumerate().for_each(
                    |(i, (_, _, object_list, _))| {
                        if object_list.list.is_empty() {
                            now_empty_index = Some(i);
                        }
                    },
                );

                if let Some(index_to_remove) = now_empty_index {
                    predicate_object_list.list.remove(index_to_remove);
                }
            }
        }

        if dependency_found {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to find dependency specification for {dependency}",
                dependency = self.dependency_name
            ))
        }
    }
}

pub struct UpdateDependency {
    pub ontology_iri: String,
    pub dependency: Dependency<SemanticVersion>,
    pub dependency_ontology_iri: String,
}

impl EditOperation for UpdateDependency {
    fn apply(&self, document: &mut TurtleDocument) -> Result<(), anyhow::Error> {
        let remove_operation = RemoveDependency {
            ontology_iri: self.ontology_iri.clone(),
            dependency_name: self.dependency.full_name.clone(),
        };
        let add_operation = AddDependency {
            ontology_iri: self.ontology_iri.clone(),
            dependency: self.dependency.clone(),
            dependency_ontology_iri: self.dependency_ontology_iri.clone(),
        };

        remove_operation.apply(document)?;
        add_operation.apply(document)?;

        Ok(())
    }
}
