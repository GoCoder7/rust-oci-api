#!/usr/bin/env bash
# release.sh — Automate oci-api crate release workflow
# Usage: bash .github/skills/release/bin/release.sh <bump_level>
#   bump_level: major | minor | patch
#
# Steps:
#   1. Validate bump level argument
#   2. Compute new version from current Cargo.toml
#   3. Run pre-release checks (test, clippy, doc)
#   4. Update Cargo.toml version
#   5. Update README.md version reference
#   6. Dry-run publish
#   7. Commit, tag, publish, push

set -euo pipefail

BUMP="${1:-}"
if [[ -z "$BUMP" || ! "$BUMP" =~ ^(major|minor|patch)$ ]]; then
  echo "Usage: $0 <major|minor|patch>"
  exit 1
fi

# Resolve project root (Cargo.toml location)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
cd "$PROJECT_ROOT"

if [[ ! -f Cargo.toml ]]; then
  echo "ERROR: Cargo.toml not found in $PROJECT_ROOT"
  exit 1
fi

# Parse current version
CURRENT=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "$BUMP" in
  major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
  minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
  patch) PATCH=$((PATCH + 1)) ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo "=== Release: v${CURRENT} → v${NEW_VERSION} ==="

# Pre-release checks
echo "--- Running tests ---"
cargo test 2>&1
echo "--- Running clippy ---"
cargo clippy 2>&1
echo "--- Building docs ---"
cargo doc --no-deps 2>&1

# Version bump
echo "--- Updating Cargo.toml version ---"
sed -i '' "s/^version = \"${CURRENT}\"/version = \"${NEW_VERSION}\"/" Cargo.toml

# Update README version reference
if grep -q "oci-api = \"${CURRENT}\"" README.md 2>/dev/null; then
  echo "--- Updating README.md version ---"
  sed -i '' "s/oci-api = \"${CURRENT}\"/oci-api = \"${NEW_VERSION}\"/" README.md
fi

# Dry-run
echo "--- Dry-run publish ---"
cargo publish --dry-run 2>&1

# Check for CHANGELOG entry
if [[ -f CHANGELOG.md ]]; then
  if ! grep -q "\[${NEW_VERSION}\]" CHANGELOG.md; then
    echo "WARNING: CHANGELOG.md does not contain an entry for [${NEW_VERSION}]"
    echo "Please update CHANGELOG.md before proceeding."
    exit 1
  fi
fi

# Commit, tag, publish
echo "--- Committing ---"
git add -A
git commit -m "release: v${NEW_VERSION}"

echo "--- Tagging ---"
git tag -a "v${NEW_VERSION}" -m "v${NEW_VERSION}"

echo "--- Publishing to crates.io ---"
cargo publish

echo "--- Pushing ---"
git push origin main
git push origin "v${NEW_VERSION}"

echo "=== Released oci-api v${NEW_VERSION} ==="
