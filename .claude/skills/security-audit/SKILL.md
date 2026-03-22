---
name: security-audit
description: Perform project-wide security audits.
---

When performing a security audit, always follow these steps:

1. **Audit Dependencies** — check for known vulnerabilities in Rust, mRuby dependencies.

2. **Static Analysis** — any potential harmful operations to host system should be avoided.

3. **Report Findings** — Document all identified risks, classify them by severity (Critical, High, Medium, Low), and provide specific remediation steps for each.
