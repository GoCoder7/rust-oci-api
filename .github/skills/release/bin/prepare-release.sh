#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: prepare-release.sh --repo-root <path> --bump <major|minor|patch> [--dry-run]

Prepares a local tagged release by checking for a clean worktree, computing the
next stable semver version, updating Cargo.toml and the README install snippet,
creating a release commit, and creating a local annotated tag. If Cargo.toml is
already at the target release version, the helper reuses the current HEAD and
creates only the tag.
EOF
}

repo_root=""
bump=""
dry_run="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      repo_root="$2"
      shift 2
      ;;
    --bump)
      bump="$2"
      shift 2
      ;;
    --dry-run)
      dry_run="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$repo_root" || -z "$bump" ]]; then
  usage >&2
  exit 1
fi

case "$bump" in
  major|minor|patch)
    ;;
  *)
    echo "Invalid bump type: $bump" >&2
    exit 1
    ;;
esac

if ! git -C "$repo_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Repository root is not a git worktree: $repo_root" >&2
  exit 1
fi

cargo_toml="$repo_root/Cargo.toml"
readme_file="$repo_root/README.md"
changelog_file="$repo_root/CHANGELOG.md"

for required_file in "$cargo_toml" "$readme_file" "$changelog_file"; do
  if [[ ! -f "$required_file" ]]; then
    echo "Missing required file: $required_file" >&2
    exit 1
  fi
done

status_output="$(git -C "$repo_root" status --short)"
if [[ -n "$status_output" ]]; then
  echo "Worktree must be clean before preparing a release." >&2
  echo "$status_output" >&2
  exit 1
fi

current_branch="$(git -C "$repo_root" rev-parse --abbrev-ref HEAD)"
latest_tag="$(git -C "$repo_root" tag --sort=-v:refname | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | head -n 1 || true)"
cargo_version="$(grep '^version = "' "$cargo_toml" | head -n 1 | sed -E 's/^version = "([^"]+)"/\1/')"
readme_version="$(grep -E 'oci-api = "' "$readme_file" | head -n 1 | sed -E 's/.*oci-api = "([^"]+)".*/\1/' || true)"

if [[ -z "$cargo_version" ]]; then
  echo "Could not determine Cargo.toml version." >&2
  exit 1
fi

if [[ -n "$readme_version" && "$readme_version" != "$cargo_version" ]]; then
  echo "README install version must match Cargo.toml before release: cargo=$cargo_version, readme=$readme_version" >&2
  exit 1
fi

release_info="$(RELEASE_BUMP="$bump" RELEASE_TAG="$latest_tag" CARGO_VERSION="$cargo_version" node <<'NODE'
const bump = process.env.RELEASE_BUMP;
const latestTag = process.env.RELEASE_TAG || "";
const cargoVersion = process.env.CARGO_VERSION || "";

const semver = /^([0-9]+)\.([0-9]+)\.([0-9]+)$/;
const tagVersion = latestTag ? latestTag.replace(/^v/, "") : "";
const baseVersion = tagVersion || cargoVersion;

if (!semver.test(baseVersion)) {
  console.error(`Cannot determine a stable semver base version from '${latestTag || cargoVersion}'.`);
  process.exit(1);
}

const match = baseVersion.match(semver);
const next = {
  major: [Number(match[1]) + 1, 0, 0],
  minor: [Number(match[1]), Number(match[2]) + 1, 0],
  patch: [Number(match[1]), Number(match[2]), Number(match[3]) + 1]
}[bump];

if (!next) {
  console.error(`Unsupported bump type '${bump}'.`);
  process.exit(1);
}

const nextVersion = next.join('.');

process.stdout.write(JSON.stringify({
  cargoVersion,
  latestTag,
  latestTagVersion: tagVersion,
  baseVersion,
  nextVersion,
  tag: `v${nextVersion}`,
  driftDetected: Boolean(tagVersion && tagVersion !== cargoVersion)
}));
NODE
)"

next_version="$(printf '%s' "$release_info" | node -e 'const fs=require("fs"); const data=JSON.parse(fs.readFileSync(0, "utf8")); process.stdout.write(data.nextVersion);')"
tag_name="$(printf '%s' "$release_info" | node -e 'const fs=require("fs"); const data=JSON.parse(fs.readFileSync(0, "utf8")); process.stdout.write(data.tag);')"

if git -C "$repo_root" rev-parse "$tag_name" >/dev/null 2>&1; then
  echo "Tag already exists: $tag_name" >&2
  exit 1
fi

commit_message="release: $tag_name"
commit_sha=""
commit_created="false"
version_already_prepared="false"
changelog_has_entry="false"
readme_version_updated="false"

if grep -q "\[$next_version\]" "$changelog_file"; then
  changelog_has_entry="true"
fi

if [[ "$next_version" == "$cargo_version" ]]; then
  version_already_prepared="true"
fi

if [[ "$dry_run" == "false" ]]; then
  if [[ "$changelog_has_entry" != "true" ]]; then
    echo "CHANGELOG.md must contain an entry for [$next_version] before preparing a release." >&2
    exit 1
  fi

  if [[ "$version_already_prepared" == "false" ]]; then
    perl -0pi -e 's/^version = "\Q'"$cargo_version"'\E"$/version = "'"$next_version"'"/m' "$cargo_toml"

    if grep -q 'oci-api = "' "$readme_file"; then
      perl -0pi -e 's/oci-api = "\Q'"$cargo_version"'\E"/oci-api = "'"$next_version"'"/g' "$readme_file"
      readme_version_updated="true"
    fi

    git -C "$repo_root" add Cargo.toml README.md
    git -C "$repo_root" commit -m "$commit_message" >/dev/null
    commit_created="true"
  fi

  git -C "$repo_root" tag -a "$tag_name" -m "$commit_message"
  commit_sha="$(git -C "$repo_root" rev-parse HEAD)"
fi

PREPARE_OUTPUT="$release_info" CURRENT_BRANCH="$current_branch" DRY_RUN="$dry_run" COMMIT_MESSAGE="$commit_message" COMMIT_SHA="$commit_sha" COMMIT_CREATED="$commit_created" VERSION_ALREADY_PREPARED="$version_already_prepared" CHANGELOG_HAS_ENTRY="$changelog_has_entry" README_VERSION_UPDATED="$readme_version_updated" REPO_ROOT="$repo_root" README_VERSION_BEFORE="$readme_version" node <<'NODE'
const info = JSON.parse(process.env.PREPARE_OUTPUT);
const output = {
  repoRoot: process.env.REPO_ROOT,
  branch: process.env.CURRENT_BRANCH,
  dryRun: process.env.DRY_RUN === 'true',
  latestTag: info.latestTag || null,
  latestTagVersion: info.latestTagVersion || null,
  cargoVersionBefore: info.cargoVersion,
  readmeVersionBefore: process.env.README_VERSION_BEFORE || null,
  baseVersion: info.baseVersion,
  driftDetected: Boolean(info.driftDetected),
  nextVersion: info.nextVersion,
  tag: info.tag,
  changelogHasEntry: process.env.CHANGELOG_HAS_ENTRY === 'true',
  commitMessage: process.env.COMMIT_MESSAGE,
  commitSha: process.env.COMMIT_SHA || null,
  commitCreated: process.env.COMMIT_CREATED === 'true',
  versionAlreadyPrepared: process.env.VERSION_ALREADY_PREPARED === 'true',
  readmeVersionUpdated: process.env.README_VERSION_UPDATED === 'true'
};

process.stdout.write(JSON.stringify(output, null, 2));
NODE
