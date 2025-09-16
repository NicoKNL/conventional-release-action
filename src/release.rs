use crate::config::Config;
use crate::file_updater::update_file_version;
use crate::scm::git::open_repository;
use git2::{ObjectType, Repository, Signature};
use semver::Version;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};

pub async fn find_previous_release_commit(
    repo: &Repository,
    config: &Config,
) -> std::result::Result<Option<git2::Oid>, Box<dyn std::error::Error>> {
    // Get all tags from the repository
    let tag_prefix = config.version.tag_prefix.as_deref().unwrap_or("");
    let tag_suffix = config.version.tag_suffix.as_deref().unwrap_or("");

    let mut versions_and_commits = Vec::new();

    repo.tag_foreach(|oid, name| {
        if let Ok(name_str) = std::str::from_utf8(name) {
            if let Some(tag_name) = name_str.strip_prefix("refs/tags/") {
                // Remove prefix and suffix to get version string
                let mut version_str = tag_name;
                if !tag_prefix.is_empty() && tag_name.starts_with(tag_prefix) {
                    version_str = &tag_name[tag_prefix.len()..];
                }
                if !tag_suffix.is_empty() && version_str.ends_with(tag_suffix) {
                    version_str = &version_str[..version_str.len() - tag_suffix.len()];
                }

                if let Ok(version) = Version::parse(version_str) {
                    versions_and_commits.push((version, oid));
                }
            }
        }
        true // Continue iteration
    })?;

    if versions_and_commits.is_empty() {
        return Ok(None);
    }

    // Sort by version and get the latest
    versions_and_commits.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(versions_and_commits.into_iter().last().map(|(_, oid)| oid))
}

pub async fn create_release_commit(
    version: &Version,
    config: &Config,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let repo = open_repository(".")?;

    // Try to find the previous release tag to chain from
    let base_commit_oid = find_previous_release_commit(&repo, config).await?;

    // Always get the current main HEAD
    let main_commit = repo.head()?.peel_to_commit()?;

    // Determine parents for the release commit
    let parents = match base_commit_oid {
        Some(oid) => {
            let previous_release_commit = repo.find_commit(oid)?;
            println!(
                "📎 Creating release with two parents: previous release {} and main {}",
                oid,
                main_commit.id()
            );
            vec![previous_release_commit, main_commit]
        }
        None => {
            println!("📎 No previous release found, basing on main branch only");
            vec![main_commit]
        }
    };

    // Update files with new version information
    if let Some(files) = &config.version.files {
        for file_config in files {
            update_file_version(file_config, version)?;
        }
    }

    // Add all updated files to the index
    let mut index = repo.index()?;
    if let Some(files) = &config.version.files {
        for file_config in files {
            if std::path::Path::new(&file_config.path).exists() {
                index.add_path(std::path::Path::new(&file_config.path))?;
            }
        }
    }
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let signature = Signature::now("Release Bot", "release@github.com")?;
    let message = format!("chore: release version {}", version);

    // Create commit with multiple parents (merge-like)
    // Detach HEAD so we don't update any branch
    repo.set_head_detached(parents[0].id())?;

    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    let commit_oid = repo.commit(
        Some("HEAD"), // Update detached HEAD
        &signature,
        &signature,
        &message,
        &tree,
        &parent_refs, // Multiple parents: previous release (if exists) and main HEAD
    )?;

    // Create the tag
    let tag_name = format!(
        "{}{}{}",
        config.version.tag_prefix.as_deref().unwrap_or(""),
        version,
        config.version.tag_suffix.as_deref().unwrap_or("")
    );

    repo.tag_lightweight(
        &tag_name,
        &repo.find_object(commit_oid, Some(ObjectType::Commit))?,
        false,
    )?;

    // Create or update major version branch (e.g., v0, v1, v2)
    let major_branch_name = format!("v{}", version.major);
    let branch_ref_name = format!("refs/heads/{}", major_branch_name);

    // Check if the branch already exists
    match repo.find_reference(&branch_ref_name) {
        Ok(mut existing_ref) => {
            // Branch exists, update it to point to new commit
            existing_ref.set_target(
                commit_oid,
                &format!("Update {} to release {}", major_branch_name, version),
            )?;
            println!(
                "📌 Updated branch {} to point to release {}",
                major_branch_name, version
            );
        }
        Err(_) => {
            // Branch doesn't exist, create it
            repo.reference(
                &branch_ref_name,
                commit_oid,
                false,
                &format!(
                    "Create {} branch for release {}",
                    major_branch_name, version
                ),
            )?;
            println!(
                "🌿 Created new branch {} for release {}",
                major_branch_name, version
            );
        }
    }

    Ok(commit_oid.to_string())
}

pub async fn push_commit_to_remote(
    commit_sha: &str,
    version: &Version,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    use git2::{Cred, PushOptions, RemoteCallbacks};

    let repo = open_repository(".")?;

    // Get the commit object
    let commit_oid = git2::Oid::from_str(commit_sha)?;

    // Create a unique temporary ref name using GitHub Actions run ID or random hash
    let unique_id = env::var("GITHUB_RUN_ID").unwrap_or_else(|_| {
        let mut hasher = DefaultHasher::new();
        commit_sha.hash(&mut hasher);
        hasher.finish().to_string()
    });

    let branch_name = format!("release-{}-{}", &commit_sha[..8], unique_id);
    let ref_name = format!("refs/heads/{}", branch_name);

    repo.reference(
        &ref_name,
        commit_oid,
        false,
        "Create temporary release branch",
    )?;

    // Set up authentication
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            Cred::userpass_plaintext("git", &token)
        } else {
            Cred::default()
        }
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // Push the temporary branch and major version branch
    let mut remote = repo.find_remote("origin")?;
    let major_branch_name = format!("v{}", version.major);
    let major_branch_ref = format!("refs/heads/{}", major_branch_name);

    let refspecs = [
        format!("{}:{}", ref_name, ref_name), // Temporary branch
        format!("{}:{}", major_branch_ref, major_branch_ref), // Major version branch
    ];
    remote.push(&refspecs, Some(&mut push_options))?;

    println!("🚀 Pushed release commit to remote branch: {}", branch_name);
    println!("🌿 Pushed major version branch: {}", major_branch_name);

    // Clean up the temporary ref locally
    let mut reference = repo.find_reference(&ref_name)?;
    reference.delete()?;

    Ok(branch_name)
}

pub async fn delete_remote_branch(
    branch_name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use git2::{Cred, PushOptions, RemoteCallbacks};

    let repo = open_repository(".")?;

    // Set up authentication
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            Cred::userpass_plaintext("git", &token)
        } else {
            Cred::default()
        }
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // Delete the remote branch by pushing an empty ref
    let mut remote = repo.find_remote("origin")?;
    let delete_refspec = format!(":refs/heads/{}", branch_name);
    remote.push(&[delete_refspec], Some(&mut push_options))?;

    println!("🗑️  Deleted temporary release branch: {}", branch_name);

    Ok(())
}
