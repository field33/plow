use git2::{Cred, CredentialType, RemoteCallbacks, Repository};
use std::path::{Path, PathBuf};

// Keeping this here for now, will be deleted later if not needed.
// const PLOW_SUBMISSION_LOCK: &str = ".plow_submission_lock";

/// A struct to represent a commit author.
#[derive(Clone, Debug)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
}

impl CommitAuthor {
    pub fn new<T: AsRef<str>>(name: T, email: T) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            email: email.as_ref().to_owned(),
        }
    }
}

#[allow(dead_code)]
/// A struct to represent a branch.
pub struct Branch {
    pub name: String,
    pub refspec: String,
    pub remote_refspec: String,
}

impl Branch {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            refspec: format!("refs/heads/{}", name.as_ref()),
            remote_refspec: format!("refs/remotes/origin/{}", name.as_ref()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn refspec(&self) -> &str {
        &self.refspec
    }

    pub fn remote_refspec(&self) -> &str {
        &self.refspec
    }
}

impl From<&str> for Branch {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for Branch {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}
impl From<&String> for Branch {
    fn from(name: &String) -> Self {
        Self::new(name)
    }
}

#[allow(dead_code)]
/// A struct to represent a plow public index repository.
pub struct PublicIndexRepository {
    pub repository: git2::Repository,
    pub cloned_branch: Branch,
    pub ssh_private_key_path: Option<PathBuf>,
    pub ssh_private_key: Option<String>,
    pub local_repository_path: PathBuf,
}

impl std::fmt::Debug for PublicIndexRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublicIndexRepository")
            .field("ssh_private_key_path", &self.ssh_private_key_path)
            .field("local_repository_path", &self.local_repository_path)
            .finish()
    }
}

impl PublicIndexRepository {
    #[allow(clippy::type_complexity)]
    /// Convenience function to re-use code in the several methods in this struct.
    ///
    /// ## Panics
    /// Panics, if we can not get the absolute path to the key.
    fn get_auth_callback(
        ssh_private_key_path: &Path,
    ) -> Box<dyn FnMut(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error>> {
        // This is to avoid introducing lifetimes, we don't care about the little overhead this will introduce here.
        // Also we need to make sure that the path is absolute because git2::Credential::ssh_key() expects it to be.
        let key_path =
            std::fs::canonicalize(PathBuf::from(ssh_private_key_path)).unwrap_or_else(|_| {
                panic!(
                    "Failed to canonicalize path: {}",
                    ssh_private_key_path.display()
                )
            });

        Box::new(
            move |_url, username_from_url: Option<&str>, _allowed_types| {
                Cred::ssh_key(
                username_from_url.expect("No username specified for remote. We right now expect the registry to lie on github with a username of `git`."),
                None,
                 &key_path,
                None)
            },
        )
    }

    #[allow(clippy::type_complexity)]
    /// Convenience function to re-use code in the several methods in this struct.
    fn get_auth_callback_with_key_from_memory(
        ssh_private_key: String,
    ) -> Box<dyn FnMut(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error>> {
        Box::new(
            move |_url, username_from_url: Option<&str>, _allowed_types| {
                Cred::ssh_key_from_memory(
                username_from_url.expect("No username specified for remote. We right now expect the registry to lie on github with a username of `git`."),
                None,
                 &ssh_private_key,
                None)
            },
        )
    }

    /// Authenticates via a private ssh key provided in the construction of the [`PublicIndexRepository`].
    pub fn auth(&self) -> anyhow::Result<RemoteCallbacks> {
        let Self {
            ssh_private_key_path,
            ssh_private_key,
            ..
        } = self;

        let mut callbacks = RemoteCallbacks::new();
        if let Some(ssh_private_key_path) = ssh_private_key_path {
            callbacks.credentials(Self::get_auth_callback(ssh_private_key_path));
            return Ok(callbacks);
        }
        // If there is no ssh_private_key_path, we assume that the ssh_private_key is in memory.
        if let Some(ssh_private_key) = ssh_private_key {
            callbacks.credentials(Self::get_auth_callback_with_key_from_memory(
                ssh_private_key.clone(),
            ));
            return Ok(callbacks);
        }
        Err(anyhow::anyhow!(
            "Couldn't find a private ssh key to authenticate with."
        ))
    }

    /// Clones the repository, if the repository exists opens it.
    ///
    /// This is also the constructor of the [`PublicIndexRepository`] struct.
    /// - Set the [`ssh_private_key_path`] field if you'd like to provide an ssh key on disk.
    /// - Set the [`ssh_private_key`] field to authenticate with a private ssh key from memory.
    ///
    /// If None set for both of them, the authentication would not complete.
    pub fn clone_or_open<T: AsRef<camino::Utf8Path>, S: AsRef<str>>(
        try_cloning_from: S,
        try_cloning_to: T,
        branch: S,
        ssh_private_key_path: Option<T>,
        ssh_private_key: Option<S>,
    ) -> anyhow::Result<Self> {
        // Attach the right authentication callback
        let mut callbacks = RemoteCallbacks::new();
        if let Some(ref ssh_private_key_path) = ssh_private_key_path {
            callbacks.credentials(Self::get_auth_callback(
                ssh_private_key_path.as_ref().as_std_path(),
            ));
        }
        // If there is no ssh_private_key_path, we assume that the ssh_private_key is in memory.
        if let Some(ref ssh_private_key) = ssh_private_key {
            callbacks.credentials(Self::get_auth_callback_with_key_from_memory(
                ssh_private_key.as_ref().to_owned(),
            ));
        }

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Build the repository with options
        let mut repo_builder = git2::build::RepoBuilder::new();
        repo_builder.fetch_options(fetch_options);

        let local_repository_path = try_cloning_to.as_ref();

        // Clone the project or open the repository
        #[allow(clippy::if_not_else)]
        let repository = if local_repository_path.exists() {
            if !local_repository_path.join(".git").exists() {
                repo_builder.clone(
                    try_cloning_from.as_ref(),
                    local_repository_path.as_std_path(),
                )?
            } else {
                Repository::open(local_repository_path)?
            }
        } else {
            repo_builder.clone(
                try_cloning_from.as_ref(),
                local_repository_path.as_std_path(),
            )?
        };

        Ok(Self {
            repository,
            cloned_branch: Branch::new(branch.as_ref()),
            ssh_private_key_path: ssh_private_key_path
                .map(|p| p.as_ref().as_std_path().to_path_buf()),
            ssh_private_key: ssh_private_key.map(|s| s.as_ref().to_owned()),
            local_repository_path: try_cloning_to.as_ref().as_std_path().to_path_buf(),
        })
    }

    /// Pulls from the remote repository.
    ///
    /// ### Order of operations
    ///
    /// - Fetch from remote
    /// - Hard reset local to `refs/remotes/origin/HEAD`
    /// - Merge remote to local (always fast forward)
    /// - Set HEAD to latest commit
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::unwrap_used)]
    #[allow(clippy::unwrap_in_result)]
    // The function will not actually panic, there is only one unwrap which is impossible to fail.
    // It is a possible index out of bounds error but our code ensures that this does not happen.
    pub fn pull_from_origin_fast_forward(&self) -> anyhow::Result<()> {
        // Attach an authentication callback
        let callbacks = self.auth()?;
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        // Always fetch all tags.
        // Perform a download and also update tips
        fetch_options.download_tags(git2::AutotagOption::All);

        let Self {
            repository,
            cloned_branch,
            ..
        } = self;

        // Get the remote HEAD commit
        let origin_head_ref = repository.find_reference("refs/remotes/origin/HEAD")?;
        let annotated_commit_ref = repository.reference_to_annotated_commit(&origin_head_ref)?;
        let annotated_commit_id = annotated_commit_ref.id();
        let commit = repository.find_commit(annotated_commit_id)?;

        // git reset --hard origin/main (remote)
        repository.reset(
            commit.as_object(),
            git2::ResetType::Hard,
            Some(git2::build::CheckoutBuilder::default().force()),
        )?;

        // Simulating a sort of git clean -f here
        let statuses = repository.statuses(None)?;
        let mut index = repository.index()?;
        let mut path_specs_to_remove = vec![];

        for i in 0..statuses.len() {
            // This unwrap will never panic.
            let status_entry = statuses.get(i).unwrap();
            let path = status_entry.path();
            let status = status_entry.status();
            // Left over file
            if status.is_wt_new() {
                if let Some(path) = path {
                    path_specs_to_remove.push(path.to_owned());
                }
            }
        }

        index.remove_all(&path_specs_to_remove, None)?;
        index.write()?;

        for path_spec in path_specs_to_remove {
            std::fs::remove_file(self.local_repository_path.join(path_spec))?;
        }

        // Get remote HEAD commit
        let mut remote = repository.find_remote("origin")?;
        remote.fetch(&[cloned_branch.name()], Some(&mut fetch_options), None)?;

        let fetch_head = repository.find_reference("FETCH_HEAD")?;
        let annotated_commit_ref = repository.reference_to_annotated_commit(&fetch_head)?;
        let annotated_commit_id = annotated_commit_ref.id();

        // Do a merge analysis
        let (merge_analysis, _) = repository.merge_analysis(&[&annotated_commit_ref])?;

        if merge_analysis.is_normal() {
            // println!("Merge analysis picked normal but we force fast forwarding anyway.");
        }

        if let Ok(mut reference) = repository.find_reference(cloned_branch.refspec()) {
            // Do the fast-forward merge
            let name = match reference.name() {
                Some(s) => s.to_owned(),
                None => String::from_utf8_lossy(reference.name_bytes()).to_string(),
            };
            let msg = format!(
                "Fast-Forward: Setting {} to id: {}",
                name, annotated_commit_id
            );
            // println!("{}", msg);
            reference.set_target(annotated_commit_id, &msg)?;
            repository.set_head(&name)?;
            repository.checkout_head(Some(
                git2::build::CheckoutBuilder::default()
                    // For some reason the force is required to make the working directory actually get updated
                    // I suspect we should be adding some logic to handle dirty working directory states
                    // but this is just an example so maybe not.
                    .force(),
            ))?;

            return Ok(());
        }
        // The branch doesn't exist so just set the reference to the
        // commit directly. Usually this is because you are pulling
        // into an empty repository.

        // This is very unlikely to happen in our case but I'd like to keep it here.
        // We'll remove it later if we never hit this path.
        repository.reference(
            cloned_branch.refspec(),
            annotated_commit_id,
            true,
            &format!(
                "Setting {} to {}",
                cloned_branch.name(),
                annotated_commit_id
            ),
        )?;
        repository.set_head(cloned_branch.refspec())?;
        repository.checkout_head(Some(
            git2::build::CheckoutBuilder::default()
                .allow_conflicts(true)
                .conflict_style_merge(true)
                .force(),
        ))?;

        Ok(())
    }
}
