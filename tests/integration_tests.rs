use std::process::Command;
use tempfile::TempDir;

#[test]
#[ignore] // This test requires Docker to be available
fn test_docker_git_ownership() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize git repo
    let init_output = Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run git init");

    assert!(init_output.status.success(), "Git init failed");

    // Create a test file and commit
    std::fs::write(temp_dir.path().join("test.txt"), "test content")
        .expect("Failed to write test file");

    Command::new("git")
        .args(&["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run git add");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git email");

    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git name");

    Command::new("git")
        .args(&["commit", "-m", "feat: test commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run git commit");

    // Build the Docker image
    let build_output = Command::new("docker")
        .args(&["build", "-t", "conventional-release-test", "."])
        .output()
        .expect("Failed to run docker build");

    assert!(
        build_output.status.success(),
        "Docker build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Run the Docker container with the test repository mounted
    let run_output = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "-v",
            &format!("{}:/github/workspace", temp_dir.path().to_string_lossy()),
            "-e",
            "DRY_RUN=true",
            "-e",
            "GITHUB_TOKEN=dummy",
            "-e",
            "GITHUB_REPOSITORY=test/test",
            "conventional-release-test",
        ])
        .output()
        .expect("Failed to run docker container");

    // The container should not fail due to git ownership issues
    println!(
        "Docker output: {}",
        String::from_utf8_lossy(&run_output.stdout)
    );
    println!(
        "Docker stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    // We expect the container to fail for other reasons (missing GitHub API, etc.)
    // but NOT due to git ownership issues
    let stderr = String::from_utf8_lossy(&run_output.stderr);
    assert!(
        !stderr.contains("not owned by current user"),
        "Git ownership error detected: {}",
        stderr
    );
    assert!(
        !stderr.contains("safe.directory"),
        "Git safe directory error detected: {}",
        stderr
    );
}

#[test]
fn test_git_safe_directory_config() {
    use conventional_release_action::scm::git::open_repository;

    // This test verifies that our git configuration works in the current environment
    let result = open_repository(".");

    assert!(
        result.is_ok(),
        "Failed to open current repository: {:?}",
        result.err()
    );

    let repo = result.unwrap();
    assert!(
        repo.head().is_ok(),
        "Should be able to access HEAD after opening repository"
    );
}
