# Release process

- [ ] Update version number in `Cargo.toml`
- [ ] Run `cargo check`
- [ ] Commit changes to `Cargo.*` with the message format `Release vN.N.N`
- [ ] Make git tag `vN.N.N`
- [ ] Run `gitchangelog`, review and make any manual adjustments as needed
- [ ] Regenerate README: `docs/generate.sh`
- [ ] Stage `CHANGES.md`, `README.md` and amend previous commit
- [ ] Force update git tag `vN.N.N`
- [ ] Push changes & tag
- [ ] Wait for builds to turn green (<https://github.com/zoni/obsidian-export/actions>)
- [ ] Run `cargo publish`
- [ ] Publish drafted release (<https://github.com/zoni/obsidian-export/releases>)
