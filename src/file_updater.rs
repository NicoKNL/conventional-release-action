use crate::config::FileUpdateConfig;
use semver::Version;
use std::path::Path;

pub fn update_file_version(
    file_config: &FileUpdateConfig,
    version: &Version,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(&file_config.path);

    if !path.exists() {
        println!("⚠️  File {} does not exist, skipping", file_config.path);
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;

    let updated_content = {
        let replacement = if let Some(template) = &file_config.template {
            template.replace("{version}", &version.to_string())
        } else {
            version.to_string()
        };

        content.replace(&file_config.marker, &replacement)
    };

    // Only write if content actually changed
    if content != updated_content {
        std::fs::write(path, updated_content)?;
        println!("📝 Updated {} version to {}", file_config.path, version);
    } else {
        println!("⚠️  No changes needed for {}", file_config.path);
    }

    Ok(())
}
