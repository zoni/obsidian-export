#!/bin/bash

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

cargo run docs "$TMPDIR"
cp "${TMPDIR}/_combined.md" README.md
