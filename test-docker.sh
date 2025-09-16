#!/bin/bash

# Test script for verifying git safe directory functionality in Docker

echo "🧪 Testing Git Safe Directory Fix"
echo "================================="

# Build the Docker image
echo "📦 Building Docker image..."
docker build -t conventional-release-action:test . || {
    echo "❌ Docker build failed"
    exit 1
}

# Create a test repository
TEST_DIR=$(mktemp -d)
echo "📁 Created test repository at: $TEST_DIR"

cd "$TEST_DIR"
git init
echo "# Test Repository" > README.md
git add .
git config user.email "test@example.com"
git config user.name "Test User"
git commit -m "feat: initial commit"

echo "✅ Test repository created"

# Test the Docker container with the repository mounted
echo "🐳 Testing Docker container..."
docker run --rm \
    -v "$TEST_DIR:/github/workspace" \
    -e DRY_RUN=true \
    -e GITHUB_TOKEN=dummy_token \
    -e GITHUB_REPOSITORY=test/test \
    conventional-release-action:test 2>&1 | tee /tmp/docker_output.log

# Check for git ownership errors
if grep -q "not owned by current user" /tmp/docker_output.log; then
    echo "❌ Git ownership error detected!"
    exit 1
elif grep -q "safe.directory" /tmp/docker_output.log; then
    echo "❌ Git safe directory error detected!"
    exit 1
else
    echo "✅ No git ownership errors detected"
fi

# Cleanup
rm -rf "$TEST_DIR"
rm -f /tmp/docker_output.log

echo "🎉 All tests passed!"
