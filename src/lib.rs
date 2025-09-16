use std::env;

pub mod bump_type;
pub mod cli;
pub mod commit;
pub mod commit_analyzer;
pub mod config;
pub mod conventional_commit;
pub mod file_updater;
pub mod output;
pub mod release;
pub mod scm;
pub mod validation;
pub mod version_manager;

use crate::cli::Args;
use crate::commit_analyzer::get_impact_from_latest_commit;
use crate::config::Config;
use crate::output::ActionOutput;
use crate::release::{create_release_commit, delete_remote_branch, push_commit_to_remote};
use crate::scm::github::GitHubClient;
use crate::validation::{should_validate_pr, validate_pr_title};
use crate::version_manager::VersionManager;

pub struct ReleaseApplication {
    config: Config,
    args: Args,
}

impl ReleaseApplication {
    pub fn new(args: Args, config: Config) -> Self {
        Self { config, args }
    }

    pub async fn run(&self) -> std::result::Result<ActionOutput, Box<dyn std::error::Error>> {
        // Change to working directory
        env::set_current_dir(&self.args.working_directory).map_err(|e| {
            format!(
                "Failed to change to working directory {:?}: {}",
                self.args.working_directory, e
            )
        })?;

        println!("🔧 Loaded configuration from {:?}", self.args.config_file);

        // Check if this is a PR and validate the title
        if should_validate_pr() {
            if let Ok(event_path) = env::var("GITHUB_EVENT_PATH") {
                validate_pr_title(&event_path).await?;
                return Ok(ActionOutput {
                    released: false,
                    version: None,
                    tag: None,
                    release_url: None,
                });
            }
        }

        // Initialize GitHub client
        let github_token = env::var("GITHUB_TOKEN")
            .map_err(|_| "GITHUB_TOKEN environment variable is required")?;
        let github_client = GitHubClient::new(github_token)?;

        // Get repository information
        let repo_info = github_client.get_repository_info().await?;
        println!("📂 Working with repository: {}", repo_info.full_name);

        // Initialize version manager
        let version_manager = VersionManager::new(&self.config, &repo_info);

        // Get current version
        let current_version = version_manager.get_current_version().await?;
        println!("📋 Current version: {}", current_version);

        // Determine version bump
        let version_bump = get_impact_from_latest_commit().await?;

        if version_bump == bump_type::BumpType::None {
            println!("ℹ️ No release needed based on the latest commit");
        }

        let new_version = version_manager.calculate_new_version(&current_version, &version_bump)?;

        if self.args.dry_run {
            println!("🚀 Proposed new version: {}", new_version);
            println!("🔍 Dry run mode - no release will be created");
            return Ok(ActionOutput {
                released: false,
                version: Some(new_version.to_string()),
                tag: None,
                release_url: None,
            });
        }

        if version_bump == bump_type::BumpType::None {
            return Ok(ActionOutput {
                released: false,
                version: Some(new_version.to_string()),
                tag: None,
                release_url: None,
            });
        }

        // Create release
        println!("🚀 Proposed new version: {}", new_version);
        let release_commit_sha = create_release_commit(&new_version, &self.config).await?;
        println!("📦 Created release commit: {}", release_commit_sha);

        // Push the commit to remote and get the branch name
        let branch_name = push_commit_to_remote(&release_commit_sha, &new_version).await?;

        let release_info = github_client
            .create_release(&repo_info, &new_version, &self.config, &release_commit_sha)
            .await?;

        // Delete the temporary remote branch after releasing
        delete_remote_branch(&branch_name).await?;

        println!("✅ Successfully created release: {}", release_info.html_url);

        Ok(ActionOutput {
            released: true,
            version: Some(new_version.to_string()),
            tag: Some(release_info.tag_name.clone()),
            release_url: Some(release_info.html_url),
        })
    }
}

// Factory function for easier testing and dependency injection
pub async fn create_release_application(
) -> std::result::Result<ReleaseApplication, Box<dyn std::error::Error>> {
    // Parse command line arguments or use environment variables (for GitHub Actions)
    let args = if env::var("GITHUB_ACTIONS").is_ok() {
        Args::from_env()
    } else {
        Args::parse()
    };

    // Load configuration
    let config = Config::load(&args.config_file)
        .map_err(|e| format!("Failed to load config from {:?}: {}", args.config_file, e))?;

    Ok(ReleaseApplication::new(args, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_args() -> Args {
        Args {
            config_file: PathBuf::from("test-config.toml"),
            dry_run: true,
            working_directory: PathBuf::from("."),
        }
    }

    fn create_test_config() -> Config {
        Config::default()
    }

    #[tokio::test]
    async fn test_release_application_creation() {
        let args = create_test_args();
        let config = create_test_config();
        let app = ReleaseApplication::new(args, config);

        // Test that the application is created correctly
        assert!(app.args.dry_run);
        assert_eq!(app.args.config_file, PathBuf::from("test-config.toml"));
    }

    #[test]
    fn test_release_application_dry_run_flag() {
        let mut args = create_test_args();
        args.dry_run = false;
        let config = create_test_config();
        let app = ReleaseApplication::new(args, config);

        assert!(!app.args.dry_run);
    }
}
