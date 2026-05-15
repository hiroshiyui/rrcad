---
name: code-review
description: Perform a project-wide code review covering security, correctness, code quality, documentation, UI/UX, and style.
---

When performing a project-wide code review:

1. Survey recent changes with `git log --oneline -20` and skim the matching diffs first.
2. Audit security, especially MCP sandboxing, Ruby prelude hardening, and FFI ownership boundaries.
3. Check correctness and memory safety, including `unsafe` blocks, OCCT bridge exception handling, and mRuby GC hazards.
4. Flag code smells such as duplicated logic, long functions, magic numbers, and avoidable `unwrap()` / `expect()` in production code.
5. Verify tests cover new DSL or geometry behavior and do not bypass single-threaded mRuby execution.
6. Check documentation for stale API, architecture, and behavioral details.
7. Review preview UI behavior, reload flow, error states, and console hygiene.
8. Confirm formatting and linting conventions are still followed.
9. Report findings grouped by category, with severity, file path, line number, description, and concrete remediation.
