# rrcad mRuby build configuration.
#
# Uses the mcp_safe gembox which restricts the available Ruby APIs to just what
# the CAD DSL needs: stdlib (String, Array, Hash, Enumerable, …) and math.
#
# The stdlib-io and metaprog gem groups are intentionally excluded to remove
# File, IO, Dir, Socket, eval(), and define_method() from the interpreter,
# eliminating the primary attack surface for prompt-injection attacks in MCP
# mode. The DSL delegates all file I/O to native Rust functions, so these gems
# are not needed in any run mode.
#
# Usage (from the repo root):
#   cd vendor/mruby && rake MRUBY_CONFIG=build_config/rrcad
# Or delete build/host/lib/libmruby.a to force a rebuild via `cargo build`.
MRuby::Build.new do |conf|
  conf.toolchain

  # Load the rrcad-safe gembox (stdlib + math only).
  conf.gembox File.join(File.dirname(__FILE__), "mcp_safe")

  conf.enable_test
end
