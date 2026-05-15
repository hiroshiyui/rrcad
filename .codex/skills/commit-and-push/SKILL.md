---
name: commit-and-push
description: Stage, commit, and push changes to the remote repository with a well-formed commit message and body.
---

When committing and pushing changes:

1. Stage only the relevant files with `git add`.
2. Commit with a Conventional Commit title.
3. Include a descriptive commit message body with the concrete change details, not just the short title.
4. Explain what changed, why, and any notable tradeoffs or test coverage in the body.
5. Push the commit to the current branch on the remote repository.
6. Verify that the push succeeded and the remote is in sync with local.
