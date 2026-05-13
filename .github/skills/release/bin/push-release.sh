#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: push-release.sh --repo-root <path> --tag <vX.Y.Z> [--remote <name>]

Publishes the crate to crates.io, then pushes the current branch tip and the
specified tag to the chosen remote.
EOF
}

repo_root=""
tag_name=""
remote_name="origin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      repo_root="$2"
      shift 2
      ;;
    --tag)
      tag_name="$2"
      shift 2
      ;;
    --remote)
      remote_name="$2"
      shift 2
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

if [[ -z "$repo_root" || -z "$tag_name" ]]; then
  usage >&2
  exit 1
fi

if ! git -C "$repo_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Repository root is not a git worktree: $repo_root" >&2
  exit 1
fi

if ! git -C "$repo_root" remote get-url "$remote_name" >/dev/null 2>&1; then
  echo "Remote does not exist: $remote_name" >&2
  exit 1
fi

if ! git -C "$repo_root" rev-parse "$tag_name" >/dev/null 2>&1; then
  echo "Tag does not exist locally: $tag_name" >&2
  exit 1
fi

status_output="$(git -C "$repo_root" status --short)"
if [[ -n "$status_output" ]]; then
  echo "Worktree must be clean before publishing a release." >&2
  echo "$status_output" >&2
  exit 1
fi

current_branch="$(git -C "$repo_root" rev-parse --abbrev-ref HEAD)"
head_commit="$(git -C "$repo_root" rev-parse HEAD)"
tag_commit="$(git -C "$repo_root" rev-list -n 1 "$tag_name")"

if [[ "$head_commit" != "$tag_commit" ]]; then
  echo "Tag $tag_name does not point to HEAD. Refusing to publish." >&2
  exit 1
fi

(cd "$repo_root" && cargo publish >/dev/null)
git -C "$repo_root" push "$remote_name" HEAD >/dev/null
git -C "$repo_root" push "$remote_name" "$tag_name" >/dev/null

REMOTE_NAME="$remote_name" CURRENT_BRANCH="$current_branch" TAG_NAME="$tag_name" HEAD_COMMIT="$head_commit" REPO_ROOT="$repo_root" node <<'NODE'
const output = {
  repoRoot: process.env.REPO_ROOT,
  remote: process.env.REMOTE_NAME,
  branch: process.env.CURRENT_BRANCH,
  tag: process.env.TAG_NAME,
  commitSha: process.env.HEAD_COMMIT,
  published: true,
  pushed: true
};

process.stdout.write(JSON.stringify(output, null, 2));
NODE
