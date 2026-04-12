#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage: ./scripts/release_version.sh <version>

Examples:
  ./scripts/release_version.sh v0.1.1
  ./scripts/release_version.sh 0.1.1

What it does:
  1. Updates version fields in:
     - package.json
     - src-tauri/tauri.conf.json
     - src-tauri/Cargo.toml
  2. Creates a release commit
  3. Pushes the current branch
  4. Creates the matching git tag
  5. Pushes the tag
EOF
}

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi

INPUT_VERSION="$1"

if [[ ! "$INPUT_VERSION" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  echo "Error: version must look like v0.1.1 or 0.1.1" >&2
  exit 1
fi

VERSION="${INPUT_VERSION#v}"
TAG="v${VERSION}"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Error: working tree must be clean before running the release script." >&2
  echo "Commit or stash your current changes first." >&2
  exit 1
fi

if git rev-parse --verify "$TAG" >/dev/null 2>&1; then
  echo "Error: local tag ${TAG} already exists." >&2
  exit 1
fi

current_branch="$(git rev-parse --abbrev-ref HEAD)"

if [[ "$current_branch" == "HEAD" ]]; then
  echo "Error: detached HEAD is not supported for release tagging." >&2
  exit 1
fi

sed_in_place() {
  local expression="$1"
  local file="$2"

  sed -E -i.bak "$expression" "$file"
  rm -f "${file}.bak"
}

echo "Updating release version to ${VERSION}..."

sed_in_place "1,10 s/^  \"version\": \"[^\"]+\"/  \"version\": \"${VERSION}\"/" "package.json"
sed_in_place "1,10 s/^  \"version\": \"[^\"]+\"/  \"version\": \"${VERSION}\"/" "src-tauri/tauri.conf.json"
sed_in_place "1,6 s/^version = \"[^\"]+\"/version = \"${VERSION}\"/" "src-tauri/Cargo.toml"

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "release: ${TAG}"

echo "Pushing branch ${current_branch}..."
git push origin HEAD

echo "Creating and pushing tag ${TAG}..."
git tag "${TAG}"
git push origin "${TAG}"

echo "Release version updated and tag pushed successfully."
