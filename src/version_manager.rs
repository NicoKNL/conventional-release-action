use semver::Version;

use crate::bump_type::BumpType;
use crate::config::Config;
use crate::scm::github::{GitHubClient, RepositoryInfo};

pub struct VersionManager<'a> {
    config: &'a Config,
    repo_info: &'a RepositoryInfo,
}

impl<'a> VersionManager<'a> {
    pub fn new(config: &'a Config, repo_info: &'a RepositoryInfo) -> Self {
        Self { config, repo_info }
    }

    pub async fn get_current_version(
        &self,
    ) -> std::result::Result<Version, Box<dyn std::error::Error>> {
        self.get_version_from_git_tags().await
    }

    pub fn calculate_new_version(
        &self,
        current: &Version,
        bump_type: &BumpType,
    ) -> std::result::Result<Version, Box<dyn std::error::Error>> {
        let mut new_version = current.clone();

        match bump_type {
            BumpType::Major => {
                new_version.major += 1;
                new_version.minor = 0;
                new_version.patch = 0;
            }
            BumpType::Minor => {
                new_version.minor += 1;
                new_version.patch = 0;
            }
            BumpType::Patch => {
                new_version.patch += 1;
            }
            BumpType::None => {
                // No version bump needed
                return Ok(current.clone());
            }
        }

        Ok(new_version)
    }

    async fn get_version_from_git_tags(
        &self,
    ) -> std::result::Result<Version, Box<dyn std::error::Error>> {
        let github_client = GitHubClient::new(
            std::env::var("GITHUB_TOKEN")
                .map_err(|_| "GITHUB_TOKEN environment variable is required")?,
        )?;

        let tags = github_client.get_tags(self.repo_info).await?;

        let tag_prefix = self.config.version.tag_prefix.as_deref().unwrap_or("");
        let tag_suffix = self.config.version.tag_suffix.as_deref().unwrap_or("");

        let mut versions = Vec::new();

        for tag in tags {
            let tag_name = &tag.name;

            // Remove prefix and suffix
            let mut version_str = tag_name.as_str();
            if !tag_prefix.is_empty() && tag_name.starts_with(tag_prefix) {
                version_str = &tag_name[tag_prefix.len()..];
            }
            if !tag_suffix.is_empty() && version_str.ends_with(tag_suffix) {
                version_str = &version_str[..version_str.len() - tag_suffix.len()];
            }

            if let Ok(version) = Version::parse(version_str) {
                versions.push(version);
            }
        }

        if versions.is_empty() {
            // No valid version tags found, use initial version
            let initial = self
                .config
                .version
                .initial_version
                .as_deref()
                .unwrap_or("0.1.0");
            return Version::parse(initial)
                .map_err(|e| format!("Invalid initial version {}: {}", initial, e).into());
        }

        // Return the highest version
        versions.sort();
        Ok(versions.into_iter().last().unwrap())
    }
}
