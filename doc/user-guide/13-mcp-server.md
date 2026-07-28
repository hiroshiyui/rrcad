# 13. MCP Server

rrcad can run as a [Model Context Protocol](https://modelcontextprotocol.io)
server, exposing the CAD engine to AI clients like Claude Desktop and
Claude Code. The client sends Ruby DSL code; rrcad evaluates it in a
sandboxed mRuby VM and returns shape metadata or export paths.

## Start it

```bash
cargo run -- --mcp
```

This starts a stdio JSON-RPC server. Configure your MCP client to launch
this command and pipe stdin/stdout — no network port is opened.

## Tools

| Tool | Input | Output |
|------|-------|--------|
| `cad_eval` | `{ "code": "..." }` | Shape type, volume, surface area, bounding box, validity |
| `cad_export` | `{ "code": "...", "format": "step" }` | `{ "path": "/tmp/rrcad_mcp/shape.step" }` |
| `cad_preview` | `{ "code": "..." }` | `{ "url": "http://localhost:<port>" }` (OS-assigned port) |
| `cad_validate` | `{ "code": "..." }` | `{ "status": "ok" }` or `{ "errors": [...] }` |

## Resources

| URI | Content |
|-----|---------|
| `rrcad://api` | Full API reference (`doc/api.md`) |
| `rrcad://examples` | All scripts from `samples/` |

## Security

The MCP server is hardened for unattended use by an AI agent:

- 30-second execution timeout (`tokio::time::timeout`).
- 2 GB address-space limit (Linux).
- Fresh mRuby VM per call — no shared state across tool invocations.
- Dangerous Kernel methods (`system`, `exec`, `fork`, …) are stripped at
  VM startup.
- All exports confined to `/tmp/rrcad_mcp/` (mode 0700); the CWD is
  changed to that directory at startup so `safe_path()` accepts the
  default export paths.
- 64 KB input size cap; null-byte filtering on incoming code.
- Printing (`puts`, `print`, `p`, `pp`) is removed, and script output is
  routed away from standard output regardless — the server's stdout carries
  the JSON-RPC responses, so stray script output would corrupt them. Return
  values from the script instead of printing them.

## Worked example: client request

A `cad_eval` call from an MCP client looks like this (the client wraps it
in JSON-RPC for you):

```ruby
# code field
b = box(10, 20, 30).fillet(2)
```

The server responds with shape metadata such as
`{ "shape_type": "solid", "volume": 5934.4, "valid": true, ... }`. To
also get an exportable file path, use `cad_export` with the desired
format.

---

[← Previous: Live Preview and REPL](12-live-preview-and-repl.md) · [Index](../user-guide.md) · [Next: Recipes →](14-recipes.md)
