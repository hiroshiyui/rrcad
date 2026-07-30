---
name: docs-engineering
description: Audit and update project documentation to stay in sync with the current codebase and development status.
---

When performing documentation engineering:

1. Survey recent changes with `git log --oneline -20` and skim the related diffs.
2. Audit the project docs for stale or inconsistent behavior, especially `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `doc/development.md`, `doc/user-guide.md` (landing page) and `doc/user-guide/*.md` (chapters), `doc/api.md`, `doc/troubleshooting.md`, `doc/TODOs.md`, `samples/README.md`, and relevant code comments.
3. Update any documentation that is stale, incomplete, or inconsistent with the codebase.
4. Remove completed items from `doc/TODOs.md` when appropriate.
5. In `doc/ROADMAP.md`, convert `- [x]` checkbox items of completed phases to plain `-` bullets, matching the other completed phases.
6. Commit documentation changes using the `commit-and-push` skill, grouped by topic.
