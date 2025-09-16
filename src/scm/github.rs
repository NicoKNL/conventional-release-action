use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
    Client,
};

use crate::config::Config;
use crate::scm::git::Tag;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub id: u64,
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub html_url: String,
    pub upload_url: String,
}

#[derive(Debug, Serialize)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub target_commitish: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepositoryInfo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: RepositoryOwner,
    pub default_branch: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepositoryOwner {
    pub login: String,
}

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: String) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| format!("Invalid GitHub token format: {}", e))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("conventional-release-action"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url: "https://api.github.com".to_string(),
        })
    }

    pub async fn get_repository_info(
        &self,
    ) -> std::result::Result<RepositoryInfo, Box<dyn std::error::Error>> {
        let repo = self.get_repository_from_env()?;
        let url = format!("{}/repos/{}", self.base_url, repo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch repository information: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API error {}: {}", status, text).into());
        }

        let repo_info = response
            .json::<RepositoryInfo>()
            .await
            .map_err(|e| format!("Failed to parse repository information: {}", e))?;

        Ok(repo_info)
    }

    pub async fn get_tags(
        &self,
        repo: &RepositoryInfo,
    ) -> std::result::Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let url = format!("{}/repos/{}/tags", self.base_url, repo.full_name);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch repository tags: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API error {}: {}", status, text).into());
        }

        let tags = response
            .json::<Vec<Tag>>()
            .await
            .map_err(|e| format!("Failed to parse repository tags: {}", e))?;

        Ok(tags)
    }

    pub async fn create_release(
        &self,
        repo: &RepositoryInfo,
        version: &Version,
        config: &Config,
        target_commit_sha: &str,
    ) -> std::result::Result<Release, Box<dyn std::error::Error>> {
        let tag_name = format!(
            "{}{}{}",
            config.version.tag_prefix.as_deref().unwrap_or(""),
            version,
            config.version.tag_suffix.as_deref().unwrap_or("")
        );

        let release_name = format!("Release {}", tag_name);
        let release_body = String::new(); // Empty body, let GitHub auto-generate if needed

        let request = CreateReleaseRequest {
            tag_name: tag_name.clone(),
            name: release_name,
            body: release_body,
            target_commitish: target_commit_sha.to_string(),
        };

        let url = format!("{}/repos/{}/releases", self.base_url, repo.full_name);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to create release: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API error {}: {}", status, text).into());
        }

        let release = response
            .json::<Release>()
            .await
            .map_err(|e| format!("Failed to parse release response: {}", e))?;

        Ok(release)
    }

    fn get_repository_from_env(&self) -> std::result::Result<String, Box<dyn std::error::Error>> {
        env::var("GITHUB_REPOSITORY")
            .map_err(|_| "GITHUB_REPOSITORY environment variable is required".into())
    }
}
