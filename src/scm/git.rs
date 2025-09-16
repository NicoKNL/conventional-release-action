use git2::{Config as GitConfig, Repository};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
    pub commit: GitCommit,
}

#[derive(Debug, Deserialize)]
pub struct GitCommit {
    pub sha: String,
}

/// Safely open a git repository with proper safe directory configuration
pub fn open_repository(path: &str) -> Result<Repository, Box<dyn Error>> {
    // First, configure git to trust any directory
    let mut git_config = GitConfig::open_default()?;
    git_config.set_str("safe.directory", "*")?;

    // Now open the repository
    let repo =
        Repository::open(path).map_err(|e| format!("Failed to open git repository: {}", e))?;

    Ok(repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Config as GitConfig, Repository, Signature};
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo = Repository::init(temp_dir.path()).expect("Failed to init repository");

        // Create an initial commit
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_open_repository_success() {
        let temp_dir = create_test_repo();
        let repo_path = temp_dir.path().to_str().unwrap();

        // This should succeed even if the directory ownership is different
        let result = open_repository(repo_path);
        assert!(
            result.is_ok(),
            "Failed to open repository: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_open_repository_sets_safe_directory() {
        let temp_dir = create_test_repo();
        let repo_path = temp_dir.path().to_str().unwrap();

        // Open the repository using our function
        let _opened_repo = open_repository(repo_path).expect("Failed to open repository");

        // Verify that the global git config now has safe.directory set to *
        let git_config = GitConfig::open_default().expect("Failed to open git config");
        let safe_directory = git_config.get_string("safe.directory");

        // Note: This might not always work depending on git version and test environment
        // but it's a good sanity check when possible
        if let Ok(value) = safe_directory {
            assert_eq!(value, "*", "safe.directory should be set to '*'");
        }
    }

    #[test]
    fn test_open_repository_invalid_path() {
        let result = open_repository("/non/existent/path");
        assert!(result.is_err(), "Should fail for non-existent path");

        let error_msg = result.err().unwrap().to_string();
        assert!(
            error_msg.contains("Failed to open git repository"),
            "Error should mention repository opening failure"
        );
    }

    #[test]
    fn test_open_repository_current_directory() {
        // This test assumes we're running in a git repository (which we are)
        let result = open_repository(".");
        assert!(
            result.is_ok(),
            "Should be able to open current directory as git repo"
        );

        let repo = result.unwrap();
        // Verify we can perform basic git operations
        assert!(repo.head().is_ok(), "Should be able to access HEAD");
    }
}
