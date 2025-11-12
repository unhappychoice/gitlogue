#!/bin/bash
# Generate CHANGELOG.md from git commit history
# Follows Conventional Commits format

set -e

CHANGELOG_FILE="CHANGELOG.md"
TEMP_FILE="${CHANGELOG_FILE}.tmp"

# Function to generate changelog section for a range of commits
function generate_section_for_commits() {
    local from_ref="$1"
    local to_ref="$2"

    # Build git log range
    if [ -z "$to_ref" ]; then
        local range="$from_ref"
    else
        local range="${to_ref}..${from_ref}"
    fi

    # Get commits and categorize them
    local features=""
    local fixes=""
    local breaking=""
    local others=""

    # Process commits
    while IFS= read -r commit; do
        if [ -z "$commit" ]; then
            continue
        fi

        local message=$(echo "$commit" | cut -d' ' -f2-)
        local commit_hash=$(echo "$commit" | cut -d' ' -f1)

        # Get repository URL from git remote
        local repo_url=$(git remote get-url origin | sed 's/\.git$//' | sed 's/git@github\.com:/https:\/\/github\.com\//')

        # Categorize commit based on conventional commit format
        case "$message" in
            feat*|feature*)
                features="${features}- ${message} ([${commit_hash:0:7}](${repo_url}/commit/${commit_hash}))\n"
                ;;
            fix*)
                fixes="${fixes}- ${message} ([${commit_hash:0:7}](${repo_url}/commit/${commit_hash}))\n"
                ;;
            *"BREAKING CHANGE"*|*"!"*)
                breaking="${breaking}- ${message} ([${commit_hash:0:7}](${repo_url}/commit/${commit_hash}))\n"
                ;;
            chore*|docs*|style*|refactor*|test*|build*|ci*)
                others="${others}- ${message} ([${commit_hash:0:7}](${repo_url}/commit/${commit_hash}))\n"
                ;;
            "Merge pull request"*)
                # Extract PR info
                local pr_info=$(echo "$message" | sed 's/Merge pull request #\([0-9]*\) from .*/PR #\1/')
                others="${others}- ${pr_info}: ${message} ([${commit_hash:0:7}](${repo_url}/commit/${commit_hash}))\n"
                ;;
            *)
                others="${others}- ${message} ([${commit_hash:0:7}](${repo_url}/commit/${commit_hash}))\n"
                ;;
        esac
    done < <(git log --oneline --no-merges "$range" 2>/dev/null || true)

    # Output categorized changes
    local has_content=false

    if [ -n "$breaking" ]; then
        echo "### 💥 BREAKING CHANGES"
        echo ""
        echo -e "$breaking"
        has_content=true
    fi

    if [ -n "$features" ]; then
        echo "### ✨ Features"
        echo ""
        echo -e "$features"
        has_content=true
    fi

    if [ -n "$fixes" ]; then
        echo "### 🐛 Bug Fixes"
        echo ""
        echo -e "$fixes"
        has_content=true
    fi

    if [ -n "$others" ]; then
        echo "### 📝 Other Changes"
        echo ""
        echo -e "$others"
        has_content=true
    fi

    if [ "$has_content" = false ]; then
        echo "- No notable changes"
        echo ""
    fi

    echo ""
}

# Get the current version from Cargo.toml
CURRENT_VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Create header
cat > "$TEMP_FILE" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

EOF

# Get all tags sorted by version (newest first)
TAGS=$(git tag -l --sort=-version:refname | grep -E '^v?[0-9]+\.[0-9]+\.[0-9]+' | head -20)

# If no tags exist, show unreleased changes
if [ -z "$TAGS" ]; then
    echo "## [Unreleased]" >> "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    generate_section_for_commits "HEAD" "" >> "$TEMP_FILE"
else
    # Get latest tag
    LATEST_TAG=$(echo "$TAGS" | head -1)

    # Check if there are unreleased changes
    UNRELEASED_COMMITS=$(git log --oneline "${LATEST_TAG}..HEAD" --pretty=format:"%s" | wc -l)

    if [ "$UNRELEASED_COMMITS" -gt 0 ]; then
        echo "## [Unreleased]" >> "$TEMP_FILE"
        echo "" >> "$TEMP_FILE"
        generate_section_for_commits "HEAD" "$LATEST_TAG" >> "$TEMP_FILE"
    fi

    # Generate sections for each tag
    PREV_TAG=""
    TAGS_ARRAY=($TAGS)
    for i in "${!TAGS_ARRAY[@]}"; do
        TAG="${TAGS_ARRAY[$i]}"
        TAG_DATE=$(git log -1 --format="%ai" "$TAG" | cut -d' ' -f1)
        CLEAN_VERSION=$(echo "$TAG" | sed 's/^v//')

        echo "## [$CLEAN_VERSION] - $TAG_DATE" >> "$TEMP_FILE"
        echo "" >> "$TEMP_FILE"

        # Get the next (older) tag for the range
        if [ $((i + 1)) -lt ${#TAGS_ARRAY[@]} ]; then
            PREV_TAG="${TAGS_ARRAY[$((i + 1))]}"
        else
            PREV_TAG=""
        fi

        generate_section_for_commits "$TAG" "$PREV_TAG" >> "$TEMP_FILE"
    done
fi

# Replace the old changelog with the new one
mv "$TEMP_FILE" "$CHANGELOG_FILE"

echo "✅ CHANGELOG.md has been generated successfully!"
echo "📄 Preview:"
echo "----------------------------------------"
head -20 "$CHANGELOG_FILE"
echo "----------------------------------------"
