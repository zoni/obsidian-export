#!/bin/sh

set -euo pipefail

cd "$(git rev-parse --show-toplevel)/book"

cargo run obsidian-src book-src
mdbook build
cp book-src/README.md ../README.md
