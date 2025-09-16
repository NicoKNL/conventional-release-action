use serde::Serialize;
use std::env;

#[derive(Serialize)]
pub struct ActionOutput {
    pub released: bool,
    pub version: Option<String>,
    pub tag: Option<String>,
    pub release_url: Option<String>,
}

pub fn output_results(output: ActionOutput) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Output for GitHub Actions
    if env::var("GITHUB_ACTIONS").is_ok() {
        if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
            let output_content = format!(
                "released={}\nversion={}\ntag={}\nrelease-url={}",
                output.released,
                output.version.as_deref().unwrap_or(""),
                output.tag.as_deref().unwrap_or(""),
                output.release_url.as_deref().unwrap_or("")
            );
            std::fs::write(output_file, output_content)
                .map_err(|e| format!("Failed to write GitHub Actions output: {}", e))?;
        }

        // Write GitHub Step Summary
        write_step_summary(&output)?;
    }

    // Also output as JSON for debugging
    println!("📊 Result: {}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn write_step_summary(
    output: &ActionOutput,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if let Ok(summary_file) = env::var("GITHUB_STEP_SUMMARY") {
        let is_pr = env::var("GITHUB_EVENT_NAME").unwrap_or_default() == "pull_request";

        let summary_content = if is_pr {
            // PR Preview Summary
            if output.released {
                format!(
                    "🔍 **Release Preview (Dry Run)**\n\n✅ **This PR would create a new release:**\n- **Proposed Version:** {}\n- **Proposed Tag:** {}\n",
                    output.version.as_deref().unwrap_or("N/A"),
                    output.tag.as_deref().unwrap_or("N/A")
                )
            } else {
                "🔍 **Release Preview (Dry Run)**\n\nℹ️ **No release would be created** - no qualifying commits found\n".to_string()
            }
        } else {
            // Release Summary
            if output.released {
                format!(
                    "🎉 **Release Created Successfully!**\n\n- **Version:** {}\n- **Tag:** {}\n- **Release URL:** {}\n",
                    output.version.as_deref().unwrap_or("N/A"),
                    output.tag.as_deref().unwrap_or("N/A"),
                    output.release_url.as_deref().unwrap_or("N/A")
                )
            } else {
                "ℹ️ **No release created** - no qualifying commits found\n".to_string()
            }
        };

        std::fs::write(summary_file, summary_content)
            .map_err(|e| format!("Failed to write GitHub Step Summary: {}", e))?;
    }

    Ok(())
}
