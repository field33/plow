//! This module is temporary to provide field consumption and dependency resolution functionality for the CLI.
//! It is an initial implementation to bring the functionality and  most likely to be rewritten very soon.

use std::str::FromStr;

use camino::Utf8PathBuf;
use colored::Colorize;
use plow_package_management::{
    package::{PackageVersion, PackageVersionWithRegistryMetadata},
    registry::{
        in_memory::InMemoryRegistry,
        index::{IndexContents, IndexedPackageVersion},
    },
    resolve::Dependency,
    version::SemanticVersion,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    config::PlowConfig, error::CliError, error::IndexSyncError::*, git::PublicIndexRepository,
};

#[derive(Serialize, Deserialize, Default)]
pub struct DifferenceQuery {
    pub existing_local_field_hashes: Vec<String>,
}

#[allow(clippy::too_many_lines)]
pub fn sync(config: &PlowConfig) -> Result<InMemoryRegistry, CliError> {
    let token = config.get_saved_api_token()?;
    let registry_url = config.get_registry_url()?;
    let private_index_sync_url = format!("{registry_url}/v1/index/private/sync");
    let client = reqwest::blocking::Client::new();

    // This is a naive and temporary solution we always get the whole private index for now.
    // This will be replaced very soon.

    let query = DifferenceQuery::default();
    let mut registry = InMemoryRegistry::default();

    println!(
        "\t{} to update the private index ..",
        "Attempting".green().bold()
    );

    let private_index_sync_response = client
        .post(private_index_sync_url)
        .header("Authorization", &format!("Basic {token}"))
        .header("Content-Type", "application/json")
        .json(&query)
        .send();

    if let Ok(response) = private_index_sync_response {
        let status = response.status();

        match status {
            StatusCode::OK => {
                if let Ok(response_body) = response.bytes() {
                    for line in response_body.split(|byte| byte == &b'\n') {
                        if line.is_empty() {
                            continue;
                        }
                        let (_name, versions) = parse(line)?;
                        for version in versions {
                            let ver = PackageVersion {
                                package_name: version.package_name.clone(),
                                version: version.version.clone().to_string(),
                            };
                            registry.packages_metadata.insert(ver, version);
                        }
                    }
                    println!(
                        "\t{} is updated successfully.",
                        "Private index".green().bold(),
                    );
                } else {
                    println!(
                        "\t{} skipping update ..",
                        "Remote private index is not reachable".red().bold(),
                    );
                }
            }
            StatusCode::UNAUTHORIZED => {
                println!(
                    "\t{} try authenticating with plow --login, skipping update ..",
                    "Unauthorized get updates from the private index"
                        .red()
                        .bold(),
                );
            }
            _ => {
                // TODO: Give real feedback and continue..
                println!(
                    "\t{} skipping update ..",
                    "Remote private index is not reachable".red().bold(),
                );
            }
        }
    } else {
        println!(
            "\t{} skipping update ..",
            "Remote private index is not reachable".red().bold(),
        );
    }

    println!(
        "\t{} to update the public index ..",
        "Attempting".green().bold(),
    );

    // TODO: Move these somewhere else?
    let clone_from = "git@github.com:field33/staging-public-registry-index.git";
    let public_index_git_repo_path = &config.index_dir.join("plow-registry-index");

    let ssh_key_path = Utf8PathBuf::from("/Users/vallahiboyle/.ssh/id_rsa");
    let repository = PublicIndexRepository::clone_or_open(
        clone_from,
        &public_index_git_repo_path,
        "main",
        Some(&&ssh_key_path),
        None,
    )
    .map_err(|err| FailedToGetRepository(err.to_string()))?;

    repository
        .pull_from_origin_fast_forward()
        .map_err(|err| FailedToGetRepository(err.to_string()))?;

    let paths = crate::utils::list_files(&public_index_git_repo_path, "json")
        .map_err(|err| FailedToReadIndexDirectory(err.to_string()))?;

    for path in paths {
        let contents =
            std::fs::read(path).map_err(|err| FailedToReadIndexDirectory(err.to_string()))?;
        let contents: IndexContents =
            serde_json::from_slice(&contents).map_err(|err| FailedToParseIndex(err.to_string()))?;
        for version in contents.versions {
            let ver = PackageVersion {
                package_name: version.name.clone(),
                version: version.version.clone(),
            };

            let mut deps = vec![];
            for dep in version.deps {
                // TODO: Corrupt index
                deps.push(
                    Dependency::<SemanticVersion>::try_from(dep.clone())
                        .map_err(|err| FailedToParseIndex(err.to_string()))?,
                );
            }

            let version = PackageVersionWithRegistryMetadata {
                package_name: version.name.clone(),
                version: SemanticVersion::from_str(&version.version)
                    .map_err(|err| FailedToParseIndex(err.to_string()))?,
                ontology_iri: version.ontology_iri,
                cksum: Some(version.cksum),
                dependencies: deps,
                private: false,
            };
            registry.packages_metadata.insert(ver, version);
        }
    }

    println!(
        "\t{} is updated successfully.",
        "Public index".green().bold(),
    );

    Ok(registry)
}

#[allow(clippy::indexing_slicing)]
fn split(haystack: &[u8], needle: u8) -> impl Iterator<Item = &[u8]> {
    struct Split<'split> {
        haystack: &'split [u8],
        needle: u8,
    }

    impl<'split> Iterator for Split<'split> {
        type Item = &'split [u8];

        fn next(&mut self) -> Option<&'split [u8]> {
            if self.haystack.is_empty() {
                return None;
            }
            let (ret, remaining) = match memchr::memchr(self.needle, self.haystack) {
                Some(pos) => (&self.haystack[..pos], &self.haystack[pos + 1..]),
                None => (self.haystack, &[][..]),
            };
            self.haystack = remaining;
            Some(ret)
        }
    }

    Split { haystack, needle }
}

const CURRENT_CACHE_VERSION: u8 = 1;
// TODO: This will come from another module later.
const INDEX_VERSION_LATEST: u32 = 1;
const NULL: u8 = 0;
const PRIVATE_VERSION: u8 = 1;

#[allow(clippy::indexing_slicing)]
pub fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> Result<&str, std::str::Utf8Error> {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8(&utf8_src[0..nul_range_end])
}

#[allow(clippy::indexing_slicing)]
fn parse(data: &[u8]) -> Result<(String, Vec<PackageVersionWithRegistryMetadata>), CliError> {
    let index_name =
        str_from_u8_nul_utf8(data).map_err(|err| FailedToParseIndex(err.to_string()))?;

    let versions_data = &data[index_name.len() + 1..];

    let (first_byte, rest) = versions_data
        .split_first()
        .ok_or_else(|| anyhow::format_err!("malformed cache"))
        .map_err(|err| FailedToParseIndex(err.to_string()))?;

    if *first_byte != CURRENT_CACHE_VERSION {
        return Err(FailedToParseIndex("cache version mismatch".to_owned()).into());
    }

    let index_version_bytes = rest
        .get(..4)
        .ok_or_else(|| anyhow::anyhow!("cache expected 4 bytes for index version"))
        .map_err(|err| FailedToParseIndex(err.to_string()))?;

    let index_version_bytes: [u8; 4] = index_version_bytes
        .try_into()
        .map_err(|_| FailedToParseIndex("index format version is corrupted".to_owned()))?;

    let index_version = u32::from_le_bytes(index_version_bytes);

    if index_version != INDEX_VERSION_LATEST {
        return Err(FailedToParseIndex("index format version mismatch".to_owned()).into());
    }

    let rest = &rest[4..];

    let mut iter = split(rest, NULL);
    if let Some(update) = iter.next() {
        std::str::from_utf8(update).map_err(|err| FailedToParseIndex(err.to_string()))?;
    } else {
        return Err(FailedToParseIndex("private index is malformed".to_owned()).into());
    };

    let mut versions = vec![];
    while let Some(version) = iter.next() {
        let version =
            std::str::from_utf8(version).map_err(|err| FailedToParseIndex(err.to_string()))?;
        let semantic_version = SemanticVersion::try_from(version)
            .map_err(|err| FailedToParseIndex(err.to_string()))?;
        match iter.next() {
            Some(version_type) => {
                let content = iter.next().ok_or_else(|| {
                    CliError::from(FailedToParseIndex("index content missing".to_owned()))
                })?;
                let IndexedPackageVersion {
                    name,
                    ontology_iri,
                    cksum,
                    deps,
                    ..
                } = serde_json::from_slice::<IndexedPackageVersion>(content)
                    .map_err(|err| FailedToParseIndex(err.to_string()))?;

                let mut dependencies = vec![];
                for dep in deps {
                    let dep = Dependency::<SemanticVersion>::try_from(dep.clone())
                        .map_err(|err| FailedToParseIndex(err.to_string()))?;
                    dependencies.push(dep);
                }

                versions.push(PackageVersionWithRegistryMetadata {
                    package_name: name,
                    version: semantic_version,
                    ontology_iri,
                    dependencies,
                    cksum: Some(cksum),
                    private: version_type[0] == PRIVATE_VERSION,
                });
            }
            _ => {
                // Unknown index type ignore..
                continue;
            }
        }
    }
    Ok((index_name.to_owned(), versions))
}
