use crate::scm::git::open_repository;
use git2::Commit as GitCommit;

use crate::bump_type::BumpType;
use crate::commit::Commit;
use std::error::Error;

pub async fn get_impact_from_latest_commit() -> Result<BumpType, Box<dyn Error>> {
    let commit = get_last_commit().await?;
    Ok(BumpType::from_conventional_commit(&commit.message))
}

async fn get_last_commit() -> Result<Commit, Box<dyn Error>> {
    let repo = open_repository(".")?;

    // Get only the HEAD commit (last commit)
    let head_commit = repo
        .head()?
        .peel_to_commit()
        .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;

    let commit = parse_commit(&head_commit)?;

    Ok(commit)
}

fn parse_commit(git_commit: &GitCommit) -> Result<Commit, Box<dyn Error>> {
    let sha = git_commit.id().to_string();
    let message = git_commit
        .message()
        .ok_or("Commit message is not valid UTF-8")?
        .to_string();

    Ok(Commit { sha, message })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scm::git::open_repository;
    use git2::{Repository, Signature};
    use tempfile::TempDir;
    use tokio;

    fn create_test_repo_with_commit(commit_message: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo = Repository::init(temp_dir.path()).expect("Failed to init repository");

        // Create an initial commit with the specified message
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, commit_message, &tree, &[])
            .unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_get_impact_from_latest_commit_feat() {
        let temp_dir = create_test_repo_with_commit("feat: add new feature");
        let original_dir = std::env::current_dir().unwrap();

        // Change to the test repository directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test that the function correctly identifies a feature commit
        let result = get_impact_from_latest_commit().await;

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), BumpType::Minor);
    }

    #[tokio::test]
    async fn test_get_impact_from_latest_commit_fix() {
        let temp_dir = create_test_repo_with_commit("fix: resolve bug");
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = get_impact_from_latest_commit().await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), BumpType::Patch);
    }

    #[tokio::test]
    async fn test_get_impact_from_latest_commit_breaking() {
        let temp_dir = create_test_repo_with_commit("feat!: breaking change");
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = get_impact_from_latest_commit().await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), BumpType::Major);
    }

    #[tokio::test]
    async fn test_get_impact_from_latest_commit_none() {
        let temp_dir = create_test_repo_with_commit("chore: update dependencies");
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = get_impact_from_latest_commit().await;

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), BumpType::None);
    }

    #[test]
    fn test_parse_commit() {
        let temp_dir = create_test_repo_with_commit("test: example commit");
        let repo = open_repository(temp_dir.path().to_str().unwrap()).unwrap();
        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();

        let result = parse_commit(&head_commit);
        assert!(result.is_ok());

        let commit = result.unwrap();
        assert_eq!(commit.message, "test: example commit");
        assert!(!commit.sha.is_empty());
    }
}
