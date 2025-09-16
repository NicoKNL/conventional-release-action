# Testing Guide for Git Safe Directory Feature

This document explains how to test the git safe directory functionality in our Docker-based GitHub Action.

## Test Categories

### 1. Unit Tests (`src/scm/git.rs`)

These tests verify that the `open_repository` function works correctly:

- **`test_open_repository_success`**: Verifies that we can open a valid git repository
- **`test_open_repository_sets_safe_directory`**: Checks that the git config is properly set
- **`test_open_repository_invalid_path`**: Ensures proper error handling for invalid paths
- **`test_open_repository_current_directory`**: Tests opening the current repository

Run these tests with:

```bash
cargo test scm::git::tests
```

### 2. Integration Tests (`src/commit_analyzer.rs`)

These tests verify that git operations work end-to-end with the safe directory configuration:

- **`test_get_impact_from_latest_commit_*`**: Test different commit types (feat, fix, breaking changes)
- **`test_parse_commit`**: Tests commit parsing functionality

Run these tests with:

```bash
cargo test commit_analyzer::tests
```

### 3. Docker Integration Tests (`tests/integration_tests.rs`)

These tests verify that the Docker container works correctly with git repositories:

- **`test_git_safe_directory_config`**: Quick verification that git config works
- **`test_docker_git_ownership`**: Full Docker test (requires Docker, marked as `#[ignore]`)

Run the basic integration test:

```bash
cargo test --test integration_tests
```

Run the Docker test (requires Docker to be running):

```bash
cargo test --test integration_tests -- --ignored
```

## Manual Testing

### Local Testing

```bash
# Build and test locally
cargo build
./target/debug/conventional-release-action --dry-run

# Test with different git repository ownership
sudo chown -R root:root /tmp/test-repo
./target/debug/conventional-release-action --dry-run
```

### Docker Testing

```bash
# Build the Docker image
docker build -t conventional-release-action .

# Test with a mounted git repository
docker run --rm \
  -v "$(pwd):/github/workspace" \
  -e DRY_RUN=true \
  -e GITHUB_TOKEN=dummy \
  -e GITHUB_REPOSITORY=test/test \
  conventional-release-action
```

### GitHub Actions Testing

The best way to test the actual git ownership issue is to create a test workflow:

```yaml
name: Test Git Ownership
on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ./
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          dry-run: true
```

## Expected Behavior

✅ **Before the fix**: You would see errors like:

```
repository path '/github/workspace' is not owned by current user distroless
```

✅ **After the fix**: The action should run successfully without ownership errors.

## Troubleshooting

If tests fail:

1. **Git not available**: Ensure git is installed in the test environment
2. **Permissions**: Some tests create temporary repositories - ensure write permissions
3. **Docker not running**: The Docker integration test requires Docker to be available
4. **Git version**: Very old git versions might not support `safe.directory` config

## Coverage

Our test suite covers:

- ✅ Basic git repository opening
- ✅ Git configuration management
- ✅ Error handling for invalid repositories
- ✅ Different commit message parsing
- ✅ Docker container execution
- ✅ Integration with actual git operations

This comprehensive testing ensures that the git safe directory feature works reliably across different environments.
