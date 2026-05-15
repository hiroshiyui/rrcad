---
name: release-engineering
description: Manage the release process, including build verification, version bumps, changelogs, tags, and GitHub releases.
---

When performing release engineering:

1. Run `./scripts/clean-build.sh` first to verify a from-scratch build.
2. Review unreleased commits since the last tag and classify the release as major, minor, or patch.
3. Update the `version` field in `Cargo.toml`.
4. Update `CHANGELOG.md` with a new release entry in Keep a Changelog format.
5. Commit the release changes with `chore: release vX.Y.Z`.
6. Create and push an annotated Git tag.
7. Create a GitHub release using the corresponding changelog section as release notes.
