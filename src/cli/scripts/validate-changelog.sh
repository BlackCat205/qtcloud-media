#!/usr/bin/env bash
set -euo pipefail

# validate-changelog.sh <version> [changelog-path]
#   version: e.g. "0.3.0" or "0.3.0-alpha.1"
#   changelog-path: 默认 "CHANGELOG.md"，可指定（如 "../CHANGELOG.md"）
#   Validates CHANGELOG exists and contains an entry for the version.

if [ $# -lt 1 ]; then
  echo "usage: validate-changelog.sh <version> [changelog-path]" >&2
  exit 1
fi

VERSION="$1"
CHANGELOG="${2:-CHANGELOG.md}"

if [ ! -f "$CHANGELOG" ]; then
  echo "CHANGELOG.md not found: $CHANGELOG" >&2
  exit 1
fi

MARKER="## [$VERSION]"
if ! grep -Fq "$MARKER" "$CHANGELOG"; then
  echo "CHANGELOG.md missing entry for version $VERSION" >&2
  exit 1
fi
