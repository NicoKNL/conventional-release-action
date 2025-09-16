use crate::conventional_commit::ConventionalCommit;
use std::env;
use std::error::Error;

pub async fn validate_pr_title(event_path: &str) -> Result<(), Box<dyn Error>> {
    let event_data = std::fs::read_to_string(event_path)?;
    let event: serde_json::Value = serde_json::from_str(&event_data)?;

    let pr_title = event["pull_request"]["title"]
        .as_str()
        .ok_or("Could not extract PR title from event")?;

    println!("🔍 Validating PR title: {}", pr_title);

    // Use ConventionalCommit parser for validation
    match ConventionalCommit::parse(pr_title) {
        Ok(commit) => {
            println!("✅ PR title follows conventional commit format");
            println!("   Type: {}", commit.commit_type);
            if let Some(scope) = &commit.scope {
                println!("   Scope: {}", scope);
            }
            println!("   Description: {}", commit.description);
            if commit.breaking_change {
                println!("   ⚠️ Breaking change detected");
            }
        }
        Err(error) => {
            eprintln!("❌ PR title does not follow conventional commit format");
            eprintln!("   Error: {}", error);
            eprintln!("Expected format: type(scope): description");
            eprintln!("Valid types: feat, fix, docs, style, refactor, perf, test, chore, build, ci, revert, security");
            eprintln!("Example: feat(auth): add user login functionality");
            std::process::exit(1);
        }
    }

    Ok(())
}

pub fn should_validate_pr() -> bool {
    if let Ok(event_name) = env::var("GITHUB_EVENT_NAME") {
        event_name == "pull_request"
    } else {
        false
    }
}
