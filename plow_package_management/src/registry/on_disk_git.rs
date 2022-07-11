//! Contains a wrapper for [`OnDiskRegistry`] for on-disk registries that are backed by a git repository.
#![allow(dead_code)]

use crate::{
    package::{PackageVersion, PackageVersionWithRegistryMetadata},
    registry::{on_disk::OnDiskRegistry, Registry},
};
use anyhow::{anyhow, bail, Error};
use git2::{build::CheckoutBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
/// Wrapper for [`OnDiskRegistry`] for on-disk registries that are backed by a git repository.
///
/// Provides additional functionality for pulling/pushing the latest state of the repository.

pub struct OnDiskGitRegistry {
    on_disk_registry: OnDiskRegistry,
    ssh_priv_key: Option<String>,
}

impl OnDiskGitRegistry {
    /// Get a [Repository] representation of our git repo.
    fn repo(&self) -> Result<Repository, Error> {
        Ok(Repository::open(&self.on_disk_registry.root_directory())?)
    }

    fn remote_callbacks_for_priv_key<'callbacks>(
        ssh_priv_key: Option<String>,
    ) -> RemoteCallbacks<'callbacks> {
        let mut callbacks = RemoteCallbacks::new();
        if let Some(ssh_priv_key) = ssh_priv_key {
            callbacks.credentials(move |_url, username_from_url, _allowed_types| {
                Cred::ssh_key_from_memory(
                    username_from_url.expect("No username specified for remote. We right now expect the registry to lie on github with a username of `git`."),
                    None,
                    &ssh_priv_key,
                    None)
            });
        }
        callbacks
    }

    #[allow(clippy::missing_panics_doc)] // TODO: Explain briefly why this function may panic
    /// Tries to initialize a local git repository from a remote.
    ///
    /// In case the directory is already present and configured with the provided remote, this does
    /// nothing, so it is save to call it repeatedly.
    pub fn initialize_from_remote<R: Into<String>, P: Into<PathBuf>>(
        remote_git_url: R,
        root_directory: P,
        ssh_priv_key: Option<String>,
    ) -> Result<Self, Error> {
        let remote_git_url: String = remote_git_url.into();
        let root_directory: PathBuf = root_directory.into();

        if root_directory.exists() {
            let repo = Repository::open(&root_directory)?;
            let origin_remote = repo.find_remote("origin")?;

            if origin_remote
                .url()
                .ok_or_else(|| anyhow!("Remote defined without URL."))?
                != remote_git_url
            {
                bail!(
                    "Git repo exists, but with wrong remote (expected `{remote_url}`)",
                    remote_url = remote_git_url
                )
            }

            return Ok(Self {
                on_disk_registry: OnDiskRegistry::new(root_directory)?,
                ssh_priv_key,
            });
        }

        fs::create_dir_all(
            root_directory.parent().ok_or_else(|| {
                anyhow!("Specified root_directory should have a parent directory.")
            })?,
        )?;
        // Set up callback to do ssh auth
        let mut callbacks = RemoteCallbacks::new();

        let ssh_priv_key_to_callback = ssh_priv_key.clone();
        if let Some(ssh_priv_key) = ssh_priv_key_to_callback {
            #[allow(clippy::unwrap_used)]
            callbacks.credentials(move |_url, username_from_url, _allowed_types| {
                Cred::ssh_key_from_memory(username_from_url.unwrap(), None, &ssh_priv_key, None)
            });
        }

        // Prepare fetch options.
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        // Prepare builder.
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        // Repository::clone(&remote_git_url, &root_directory)?;
        builder.clone(&remote_git_url, &root_directory)?;

        Ok(Self {
            on_disk_registry: OnDiskRegistry::new(root_directory)?,
            ssh_priv_key,
        })
    }

    /// Initializes a [`OnDiskGitRepository`], assuming that it is set up with a remote repository.
    ///
    /// Only checks whether there is a git repository in the provided `root_directory`.
    pub fn new_assume_initialized<P: Into<PathBuf>>(
        root_directory: P,
        ssh_priv_key: Option<String>,
    ) -> Result<Self, Error> {
        let root_directory: PathBuf = root_directory.into();

        // Check that there is a repository at the specified directory.
        Repository::open(&root_directory)?;

        Ok(Self {
            on_disk_registry: OnDiskRegistry::new(root_directory)?,
            ssh_priv_key,
        })
    }

    /// Updates the registry by pulling the latest changes from remote.
    pub fn update_from_remote(&mut self) -> Result<(), Error> {
        let remote_name = "origin";
        let remote_branch_name = "main";
        let repo = self.repo()?;

        let callbacks = Self::remote_callbacks_for_priv_key(self.ssh_priv_key.clone());
        repo.find_remote(remote_name)?.fetch(
            &[remote_branch_name],
            Some(FetchOptions::default().remote_callbacks(callbacks)),
            None,
        )?;

        let refname = format!(
            "refs/remotes/{remote}/{branch}",
            remote = remote_name,
            branch = remote_branch_name
        );

        repo.set_head(&refname)?;
        repo.checkout_head(Some(CheckoutBuilder::default().force()))?;

        Ok(())
    }
}

impl Registry for OnDiskGitRegistry {
    fn all_available_versions_of_a_package(
        &self,
        package_namespace_and_name: String,
    ) -> Vec<PackageVersionWithRegistryMetadata> {
        self.on_disk_registry
            .list_all_package_versions()
            .unwrap_or_default()
            .into_iter()
            .filter(|package_version| package_version.package_name == package_namespace_and_name)
            .collect()
    }
    fn get_package_version_metadata(
        &self,
        package_version: &PackageVersion,
    ) -> Result<PackageVersionWithRegistryMetadata, Error> {
        self.on_disk_registry
            .get_package_version_metadata(package_version)
    }

    fn retrieve_package(&self, package: &PackageVersion) -> Result<Vec<u8>, Error> {
        self.on_disk_registry.retrieve_package(package)
    }

    fn submit_package(
        &self,
        _file_contents: &str,
    ) -> Result<PackageVersionWithRegistryMetadata, Error> {
        unimplemented!()
    }
}
