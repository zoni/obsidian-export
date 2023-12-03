#!/usr/bin/env bash

set -euo pipefail

get_next_version_number() {
	DATEPART=$(date +%y.%-m)
	ITERATION=0

	while true; do
		VERSION_STRING="${DATEPART}.${ITERATION}"
		if git rev-list "v$VERSION_STRING" > /dev/null 2>&1; then
			((ITERATION++))
		else
			echo "$VERSION_STRING"
			return
		fi
	done
}

git add .
if ! git diff-index --quiet HEAD; then
	printf "Working directory is not clean. Please commit or stash your changes.\n"
	exit 1
fi

VERSION=$(get_next_version_number)
git tag "v${VERSION}"

git cliff --latest --prepend CHANGELOG.md > /dev/null
${EDITOR:-vim} CHANGELOG.md
docs/generate.sh

sed -i -E "s/^version = \".+\"$/version = \"${VERSION}\"/" Cargo.toml
cargo check

git add .
# There are likely trailing whitespace changes in the changelog, but a single
# run of pre-commit will fix these automatically.
pre-commit run || git add .

git commit --message "Release v${VERSION}"
git tag "v${VERSION}" --force

printf "\n\nSuccessfully created release %s\n" "v${VERSION}"
printf "\nYou'll probably want to continue with:\n"
printf "\tgit push origin main\n"
printf "\tgit push origin %s\n" "v${VERSION}"
