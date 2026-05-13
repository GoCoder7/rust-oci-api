#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF' >&2
[release] The one-shot release.sh workflow is deprecated.

Use the staged release helpers instead:

1. Preview the release plan:
   bash ./.github/skills/release/bin/prepare-release.sh --repo-root . --bump <major|minor|patch> --dry-run

2. Prepare the local release commit and tag:
   bash ./.github/skills/release/bin/prepare-release.sh --repo-root . --bump <major|minor|patch>

3. After explicit approval, publish and push:
   bash ./.github/skills/release/bin/push-release.sh --repo-root . --tag <vX.Y.Z>
EOF

exit 1
