mod catalog;

use crate::{
    config::PlowConfig,
    error::CliError,
    error::FieldAccessError::*,
    error::ProtegeSubcommandError::*,
    feedback::{field_info, Feedback},
    resolve::resolve,
    subcommand::protege::catalog::CatalogFile,
};

use camino::{Utf8Path, Utf8PathBuf};
use clap::{arg, App, AppSettings, ArgMatches, Command};
use colored::*;
use harriet::Whitespace;
use harriet::{IRIReference, ObjectList};
use harriet::{Literal, Object};
use plow::manifest::FieldManifest;
use plow_linter::lints::all_lints;
use plow_package_management::{
    package::{RetrievedPackageSet, RetrievedPackageVersion},
    registry::Registry,
};

use sha2::{Digest, Sha256};

use super::lint::lint_file;

pub struct SuccessfulProtege;
impl Feedback for SuccessfulProtege {
    fn feedback(&self) {
        println!(
            "\t{} successfully opened in protege.",
            "Field".green().bold(),
        );
    }
}

pub fn attach_as_sub_command() -> App<'static> {
    Command::new("protege")
        .about("Opens a field in protege.")
        .arg(arg!([FIELD_PATH]))
        .setting(AppSettings::ArgRequiredElseHelp)
}

#[allow(clippy::as_conversions)]
pub fn run_command(sub_matches: &ArgMatches, config: &PlowConfig) -> Box<dyn Feedback + 'static> {
    match run_command_flow(sub_matches, config) {
        Ok(feedback) => Box::new(feedback) as Box<dyn Feedback>,
        Err(feedback) => Box::new(feedback) as Box<dyn Feedback>,
    }
}

#[allow(clippy::too_many_lines)]
pub fn run_command_flow(
    sub_matches: &clap::ArgMatches,
    config: &PlowConfig,
) -> Result<impl Feedback, CliError> {
    // let workspace_root = config.working_dir.fail_if_not_under_a_workspace()?;
    // let workspace_root = config.working_dir.path.clone();
    let field_file_path = sub_matches
        .get_one::<String>("FIELD_PATH")
        .ok_or(NoFieldProvidedToOpenInProtege)?;

    let field_file_path = camino::Utf8PathBuf::from(field_file_path);
    field_info(&field_file_path)?;

    if field_file_path.exists() {
        lint_file(field_file_path.as_ref(), all_lints())?;

        let registry = crate::sync::sync(config)?;

        let root_field_contents = std::fs::read_to_string(&field_file_path).map_err(|_| {
            CliError::from(FailedToFindFieldAtPath {
                field_path: field_file_path.to_string(),
            })
        })?;

        let root_field_manifest = FieldManifest::new(&root_field_contents).map_err(|_| {
            CliError::from(FailedToReadFieldManifest {
                field_path: field_file_path.to_string(),
            })
        })?;

        match root_field_manifest.dependencies_stated_in_owl_imports() {
            Ok((
                mut object_list,
                stated_dep_names_in_owl_imports,
                base_iri_beginning,
                statements,
            )) => {
                if let Some(lock_file) = resolve(
                    config,
                    &root_field_contents,
                    &root_field_manifest,
                    true,
                    &registry as &dyn Registry,
                )? {
                    // Leave an empty line in between.
                    println!();
                    println!("\t{}", "Dependencies".bold().green());

                    lock_file
                        .locked_dependencies
                        .packages
                        .iter()
                        .for_each(|package_version| {
                            println!(
                                "\t\t{} {}",
                                package_version.package_name.bold(),
                                package_version.version
                            );
                        });

                    // Only names of resolved dependencies.
                    let resolved_dep_names = lock_file
                        .locked_dependencies
                        .packages
                        .iter()
                        .map(|p| p.package_name.clone())
                        .collect::<Vec<_>>();

                    // Collect deps which does not exist as owl imports.
                    // Collect deps to be deleted from owl imports.
                    let (deps_to_add_to_owl_imports, deps_to_remove_from_owl_imports) =
                        resolved_dep_names.iter().fold(
                            (vec![], vec![]),
                            |mut acc, package_name| {
                                if !stated_dep_names_in_owl_imports.contains(package_name) {
                                    acc.0.push(package_name.clone());
                                }
                                for dep_name in &stated_dep_names_in_owl_imports {
                                    if !resolved_dep_names.contains(dep_name) {
                                        acc.1.push(dep_name.clone());
                                    }
                                }
                                acc
                            },
                        );

                    // Inject necessary owl:imports
                    for to_add in deps_to_add_to_owl_imports {
                        if object_list.list.is_empty() {
                            object_list.list.push(make_owl_imports_object(
                                &to_add,
                                &base_iri_beginning,
                                true,
                            ));
                        } else {
                            object_list.list.push(make_owl_imports_object(
                                &to_add,
                                &base_iri_beginning,
                                false,
                            ));
                        }
                    }

                    // Delete unnecessary owl:imports

                    object_list.list = object_list
                        .list
                        .iter()
                        .cloned()
                        .filter(|(_, _, object)| match object {
                            Object::IRI(harriet::IRI::IRIReference(iri_ref)) => {
                                let iri_literal = iri_ref.iri.to_string();
                                let iris = iri_literal.split('/').collect::<Vec<&str>>();
                                let mut it = iris.iter().rev();
                                it.next();
                                // TODO: These unwraps may indeed fail but it is unlikely to happen.
                                // They will be addressed in the refactoring.
                                #[allow(clippy::unwrap_used)]
                                let name = it.next().unwrap().to_owned();
                                // TODO: These unwraps may indeed fail but it is unlikely to happen.
                                // They will be addressed in the refactoring.
                                #[allow(clippy::unwrap_used)]
                                let namespace = it.next().unwrap().to_owned();
                                let dep_name = format!("{namespace}/{name}");
                                !deps_to_remove_from_owl_imports.contains(&dep_name)
                            }
                            _ => true,
                        })
                        .collect::<Vec<_>>();

                    // Here the owl imports are updated.
                    // We just need to add them to the existing statements and re-serialize.

                    root_field_manifest.update_owl_imports_and_serialize(
                        object_list,
                        statements,
                        &field_file_path,
                    );

                    // TODO: These unwraps may indeed fail but it is unlikely to happen.
                    // They will be addressed in the refactoring.
                    #[allow(clippy::unwrap_used)]
                    let dependency_information = lock_file
                        .locked_dependencies
                        .packages
                        .iter()
                        .map(|package_version| {
                            let metadata = registry
                                .get_package_version_metadata(package_version)
                                .unwrap();
                            let name = format!("{}.ttl", metadata.cksum.unwrap());
                            let ontology_iri = metadata.ontology_iri.unwrap();

                            // TODO: When types are updated with the upcoming refactoring this will be updated also
                            RetrievedPackageVersion {
                                ontology_iri,
                                package: package_version.clone(),
                                file_path: config.field_cache_dir.join(name),
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut set = RetrievedPackageSet {
                        packages: dependency_information,
                    };

                    let root_path = &field_file_path;

                    let (_protege_workspace_dir, symlinked_field_path) =
                        mirror_field_to_protege_workspace(root_path).map_err(|err| {
                            CliError::from(FailedToPrepareProtegeWorkspace(err.to_string()))
                        })?;

                    #[allow(clippy::unwrap_used)]
                    let current_protege_workspace = symlinked_field_path.parent().unwrap();

                    let deps_path = current_protege_workspace.join("deps");
                    if !deps_path.exists() {
                        std::fs::create_dir_all(&deps_path).map_err(|err| {
                            CliError::from(FailedToPrepareProtegeWorkspace(err.to_string()))
                        })?;
                    }
                    for package in &mut set.packages {
                        #[allow(clippy::unwrap_used)]
                        let dep_path_in_protege_workspace =
                            deps_path.join(&package.file_path.file_name().unwrap());

                        std::fs::copy(&package.file_path, &dep_path_in_protege_workspace).map_err(
                            |err| CliError::from(FailedToPrepareProtegeWorkspace(err.to_string())),
                        )?;
                        package.file_path = dep_path_in_protege_workspace;
                    }
                    // Generate catalog file
                    CatalogFile::generate(current_protege_workspace, &set).map_err(|err| {
                        CliError::from(FailedToPrepareProtegeWorkspace(err.to_string()))
                    })?;

                    // Open in protege
                    open::that(symlinked_field_path).map_err(|err| {
                        CliError::from(FailedToOpenProtegeApplication(err.to_string()))
                    })?;
                }
            }
            Err(statements) => {
                if let Some(lock_file) = resolve(
                    config,
                    &root_field_contents,
                    &root_field_manifest,
                    true,
                    &registry as &dyn Registry,
                )? {
                    // Leave an empty line in between.
                    println!();
                    println!("\t{}", "Dependencies".bold().green());

                    lock_file
                        .locked_dependencies
                        .packages
                        .iter()
                        .for_each(|package_version| {
                            println!(
                                "\t\t{} {}",
                                package_version.package_name.bold(),
                                package_version.version
                            );
                        });

                    // Only names of resolved dependencies.
                    let resolved_dep_names = lock_file
                        .locked_dependencies
                        .packages
                        .iter()
                        .map(|p| p.package_name.clone())
                        .collect::<Vec<_>>();

                    let base_iri = root_field_manifest.ontology_iri();
                    let base_iri_parts = base_iri.split('/').collect::<Vec<_>>();
                    let base_iri_beginning = base_iri_parts[..base_iri_parts.len() - 3].join("/");

                    let mut object_list = ObjectList { list: vec![] };

                    for to_add in &resolved_dep_names {
                        if object_list.list.is_empty() {
                            object_list.list.push(make_owl_imports_object(
                                to_add,
                                &base_iri_beginning,
                                true,
                            ));
                        } else {
                            object_list.list.push(make_owl_imports_object(
                                to_add,
                                &base_iri_beginning,
                                false,
                            ));
                        }
                    }

                    let predicate = crate::subcommand::init::field::make_predicate_object(
                        "owl",
                        "imports",
                        object_list,
                    );

                    root_field_manifest.create_owl_imports_and_serialize(
                        predicate,
                        statements,
                        &field_file_path,
                    );

                    // TODO: These unwraps may indeed fail but it is unlikely to happen.
                    // They will be addressed in the refactoring.
                    #[allow(clippy::unwrap_used)]
                    let dependency_information = lock_file
                        .locked_dependencies
                        .packages
                        .iter()
                        .map(|package_version| {
                            let metadata = registry
                                .get_package_version_metadata(package_version)
                                .unwrap();
                            let name = format!("{}.ttl", metadata.cksum.unwrap());
                            let ontology_iri = metadata.ontology_iri.unwrap();

                            // TODO: When types are updated with the upcoming refactoring this will be updated also
                            RetrievedPackageVersion {
                                ontology_iri,
                                package: package_version.clone(),
                                file_path: config.field_cache_dir.join(name),
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut set = RetrievedPackageSet {
                        packages: dependency_information,
                    };

                    let root_path = &field_file_path;

                    let (_protege_workspace_dir, symlinked_field_path) =
                        mirror_field_to_protege_workspace(root_path).map_err(|err| {
                            CliError::from(FailedToPrepareProtegeWorkspace(err.to_string()))
                        })?;

                    #[allow(clippy::unwrap_used)]
                    let current_protege_workspace = symlinked_field_path.parent().unwrap();

                    let deps_path = current_protege_workspace.join("deps");
                    if !deps_path.exists() {
                        std::fs::create_dir_all(&deps_path).map_err(|err| {
                            CliError::from(FailedToPrepareProtegeWorkspace(err.to_string()))
                        })?;
                    }
                    for package in &mut set.packages {
                        #[allow(clippy::unwrap_used)]
                        let dep_path_in_protege_workspace =
                            deps_path.join(&package.file_path.file_name().unwrap());

                        std::fs::copy(&package.file_path, &dep_path_in_protege_workspace).map_err(
                            |err| CliError::from(FailedToPrepareProtegeWorkspace(err.to_string())),
                        )?;
                        package.file_path = dep_path_in_protege_workspace;
                    }
                    // Generate catalog file
                    CatalogFile::generate(current_protege_workspace, &set).map_err(|err| {
                        CliError::from(FailedToPrepareProtegeWorkspace(err.to_string()))
                    })?;

                    dbg!("COMES");

                    // Open in protege
                    open::that(symlinked_field_path).map_err(|err| {
                        CliError::from(FailedToOpenProtegeApplication(err.to_string()))
                    })?;
                }
            }
        }
        return Ok(SuccessfulProtege);
    }

    Err(FailedToFindFieldAtPath {
        field_path: field_file_path.into(),
    }
    .into())
}

pub fn hash_field_path(field_path: &Utf8Path) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(field_path.to_string().as_bytes());
    format!("{:X}", sha256.finalize())
}

// pub struct ProtegeWorkspace {
//     pub workspace_dir: Utf8PathBuf,
//     pub field_path: Utf8PathBuf,
// }

/// Creates a directory in our workspace directory based on the provided file.
///
/// The workspace directory will contain a symlink to the original file (so changes propagate
/// back to the original file), and can also be used to place e.g. a catalog file and resolved
/// dependencies for the file.
pub fn mirror_field_to_protege_workspace(
    field_path: &Utf8Path,
) -> Result<(Utf8PathBuf, Utf8PathBuf), anyhow::Error> {
    let field_path_hash: String = hash_field_path(field_path);
    let protege_workspace_dir = Utf8PathBuf::from_path_buf(
        dirs::document_dir()
            .ok_or_else(|| anyhow::anyhow!("No document dir known for platform"))?
            .join("plow")
            .join("protege_workspaces")
            .join(field_path_hash),
    )
    .map_err(|_| anyhow::anyhow!("Path utf8 conversion error"))?;

    let symlinked_field_path = protege_workspace_dir.join(
        field_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Provided path is not a file"))?,
    );

    if protege_workspace_dir.exists() {
        std::fs::remove_dir_all(&protege_workspace_dir)?;
    }
    std::fs::create_dir_all(&protege_workspace_dir)?;

    if symlinked_field_path.exists() {
        std::fs::remove_file(&symlinked_field_path)?;
    }
    std::fs::hard_link(&field_path, &symlinked_field_path)?;

    Ok((protege_workspace_dir, symlinked_field_path))
}

fn make_owl_imports_object<'func>(
    dep_name: &str,
    base_iri_root: &str,
    first: bool,
) -> (
    Option<Whitespace<'func>>,
    Option<Whitespace<'func>>,
    Object<'func>,
) {
    let iri_literal = format!("{}/{}/", base_iri_root, dep_name);
    (
        if first {
            None
        } else {
            Some(Whitespace {
                whitespace: " ".into(),
            })
        },
        Some(Whitespace {
            whitespace:
                "\n                                                                           "
                    .into(),
        }),
        Object::IRI(harriet::IRI::IRIReference(IRIReference {
            iri: iri_literal.into(),
        })),
    )
}
