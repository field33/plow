use std::io::Cursor;

use colored::Colorize;

use plow_package_management::{
    lock::LockFile, package::OrganizationToResolveFor, registry::Registry, resolve::Dependency,
    version::SemanticVersion,
};
use reqwest::StatusCode;

use crate::{
    config::PlowConfig,
    error::CliError,
    error::{FieldDownloadError::*, ResolveError::*},
    manifest::FieldManifest,
};

#[allow(clippy::missing_panics_doc)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::too_many_lines)]
pub fn resolve(
    config: &PlowConfig,
    _: &str,
    root_field_manifest: &FieldManifest,
    respect_existing_lock_file: bool,
    registry: &dyn Registry,
) -> Result<Option<LockFile>, CliError> {
    let workspace_root = config.get_workspace_root().ok();

    println!(
        "\t{} to resolve dependencies of {} ..",
        "Attempting".green().bold(),
        root_field_manifest
            .field_namespace_and_name()
            .unwrap()
            .bold()
    );

    #[allow(clippy::unwrap_used)]
    // We run this one after linting.
    if let Some(deps) = root_field_manifest.field_dependency_literals() {
        let deps = deps
            .into_iter()
            .map(|dep| Dependency::<SemanticVersion>::try_from(dep.as_str()).unwrap())
            .collect::<Vec<_>>();

        // TODO: Don't forget to write the initial package to the lock file also.
        // Needs to get the workspace root and check for lock file.
        // Either in dep resolver.
        // Read it and rewrite it after resolution.
        // Update command needs to be created to update the lock file.

        // Here.. we extend the entry with the initial package we resolve the deps for. Let's give it a try.

        let entry = OrganizationToResolveFor {
            package_name: "@root/root".to_owned(),
            package_version: SemanticVersion::default(),
            dependencies: deps,
        };

        let locked_and_resolved = LockFile::lock_with_registry(
            entry,
            registry,
            workspace_root,
            respect_existing_lock_file,
        )
        .map_err(|err| CliError::from(FailedToResolveDependencies(err.to_string())))?;

        let metadatas = locked_and_resolved
            .locked_dependencies
            .packages
            .iter()
            .map(|package_version| {
                // Dependency resolution would catch this earlier.
                // Unwrap is fine.
                registry
                    .get_package_version_metadata(package_version)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let cksums = metadatas
            .iter()
            .filter_map(|metadata| metadata.cksum.clone())
            .collect::<Vec<_>>();

        let paths = crate::utils::list_files(&config.field_cache_dir, "ttl").map_err(|err| {
            CliError::from(FailedToReadFieldCache {
                reason: err.to_string(),
            })
        })?;

        let stems = paths
            .iter()
            .map(|path| path.file_stem().unwrap())
            .collect::<Vec<&str>>();

        // Cache check.
        let cksums_to_download = cksums
            .iter()
            .filter(|cksum| stems.contains(&cksum.as_str()))
            .collect::<Vec<_>>();

        let client = reqwest::blocking::Client::new();
        let registry_url = config.get_registry_url()?;
        let token = config.get_saved_api_token()?;

        for download in cksums_to_download {
            println!("\t{} to download field contents ..", "Attempting".bold());

            let signed_url_request = format!("{registry_url}/v1/artifact/signed-url/{download}");

            let signed_url_response = client
                .get(signed_url_request)
                .header("Authorization", &format!("Basic {token}"))
                .send()
                .map_err(|err| {
                    CliError::from(FailedToDownloadAndCacheField {
                        reason: format!(
                            "Attempt of retrieving a download link for the field failed. Error: {err}"
                        ),
                    })
                })?;

            let status = signed_url_response.status();
            if !status.is_success() {
                if status == StatusCode::NOT_FOUND {
                    return Err(CliError::from(FailedToDownloadAndCacheField {
                        reason: "The field was not found in registry.".to_owned(),
                    }));
                }
                return Err(CliError::from(FailedToDownloadAndCacheField {
                    reason: format!("Download request failed with status code: {status}"),
                }));
            }

            let response_body_value =
                signed_url_response
                    .json::<serde_json::Value>()
                    .map_err(|_| {
                        CliError::from(FailedToDownloadAndCacheField {
                            reason: "Corrupt download link retrieved.".to_owned(),
                        })
                    })?;

            let signed_url = response_body_value
                .get("data")
                .ok_or_else(|| {
                    CliError::from(FailedToDownloadAndCacheField {
                        reason: "Corrupt download link retrieved.".to_owned(),
                    })
                })?
                .as_object()
                .ok_or_else(|| {
                    CliError::from(FailedToDownloadAndCacheField {
                        reason: "Corrupt download link retrieved.".to_owned(),
                    })
                })?
                .get("url")
                .ok_or_else(|| {
                    CliError::from(FailedToDownloadAndCacheField {
                        reason: "Corrupt download link retrieved.".to_owned(),
                    })
                })?
                .as_str()
                .ok_or_else(|| {
                    CliError::from(FailedToDownloadAndCacheField {
                        reason: "Corrupt download link retrieved.".to_owned(),
                    })
                })?;

            let download_result = client.get(signed_url).send().map_err(|err| {
                CliError::from(FailedToDownloadAndCacheField {
                    reason: format!("Download link is invalid. Error: {err}"),
                })
            })?;

            let mut file =
                std::fs::File::create(&config.field_cache_dir.join(format!("{download}.ttl")))
                    .map_err(|err| {
                        CliError::from(FailedToDownloadAndCacheField {
                            reason: format!(
                                "Couldn't write retrieved field to the filesystem. Error: {err}"
                            ),
                        })
                    })?;

            let mut content = Cursor::new(download_result.bytes().map_err(|err| {
                CliError::from(FailedToDownloadAndCacheField {
                    reason: format!(
                        "Couldn't write retrieved field to the filesystem. Error: {err}"
                    ),
                })
            })?);

            std::io::copy(&mut content, &mut file).map_err(|err| {
                CliError::from(FailedToDownloadAndCacheField {
                    reason: format!(
                        "Couldn't write retrieved field to the filesystem. Error: {err}"
                    ),
                })
            })?;
            println!("\t{} successful.", "Download".green().bold());
        }

        println!(
            "\t{} resolved dependencies of {} ..",
            "Successfully".green().bold(),
            root_field_manifest
                .field_namespace_and_name()
                .unwrap()
                .bold()
        );

        return Ok(Some(locked_and_resolved));
    }

    println!("\t{} to resolve.", "No dependencies".yellow(),);

    Ok(None)
}
