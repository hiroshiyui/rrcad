# rrcad User Guide

**rrcad** is a Ruby DSL for 3D parametric CAD. You write `.rb` scripts; an
embedded mRuby VM executes them; Rust bindings call OpenCASCADE (OCCT) for
exact BRep geometry. The result is industrial-grade solids exportable to
STEP, STL, glTF, OBJ, SVG, and DXF.

This guide is written for CAD engineers — mechanical designers, fixture
makers, 3D-printing hobbyists, and anyone who builds parts by *typing* rather
than dragging a mouse. Each chapter is task-oriented: start with the problem,
introduce the DSL, finish with a worked example you can copy into a `.rb`
file and run.

## Start here

New to rrcad? Read these in order:

1. [Getting Started](user-guide/01-getting-started.md) — install, run your
   first script, learn the CLI modes and the optional `rrcad.toml`.
2. [Modeling Basics](user-guide/02-modeling-basics.md) — units, primitive
   solids, transforms, and boolean operations.
3. [Sketches and Profiles](user-guide/03-sketches-and-profiles.md) — 2D
   shapes, splines, and constraint-driven sketches.
4. [Features and Modifiers](user-guide/04-features-and-modifiers.md) —
   extrude, revolve, sweep, fillet, chamfer, shell.

## All chapters

| # | Chapter | What's inside |
|---|---------|---------------|
| 1 | [Getting Started](user-guide/01-getting-started.md) | Install, first script, CLI modes, `rrcad.toml` |
| 2 | [Modeling Basics](user-guide/02-modeling-basics.md) | Units, primitives, transforms, booleans |
| 3 | [Sketches and Profiles](user-guide/03-sketches-and-profiles.md) | 2D faces, splines, constraint sketches |
| 4 | [Features and Modifiers](user-guide/04-features-and-modifiers.md) | Extrude, revolve, sweep, fillet, chamfer, shell, offset |
| 5 | [Part Design](user-guide/05-part-design.md) | Pad / pocket, datum planes, holes, threads, fastener bodies |
| 6 | [Topology and Selectors](user-guide/06-topology-and-selectors.md) | Face / edge selectors, named topology, GD&T |
| 7 | [Patterns and Surfaces](user-guide/07-patterns-and-surfaces.md) | Linear / polar / grid patterns, ruled / fill surfaces, slice |
| 8 | [Assemblies](user-guide/08-assemblies.md) | Assembly DSL, mate / distance / axis / angle constraints, color |
| 9 | [Inspection and CAM Checks](user-guide/09-inspection-and-cam.md) | Validate, history, centroid / inertia, manufacturability checks |
| 10 | [Import and Export](user-guide/10-import-export.md) | Formats, SVG / DXF views, GLB tessellation, GD&T frames |
| 11 | [Parametric Design and Batch Export](user-guide/11-parametric-and-batch.md) | `param`, CLI overrides, CSV design tables, multi-file projects |
| 12 | [Live Preview and REPL](user-guide/12-live-preview-and-repl.md) | Browser viewer, viewer controls, interactive REPL |
| 13 | [MCP Server](user-guide/13-mcp-server.md) | JSON-RPC tools, resources, security |
| 14 | [Recipes](user-guide/14-recipes.md) | Cross-cutting common patterns |
| 15 | [Troubleshooting](user-guide/15-troubleshooting.md) | Reading errors, build failures, geometry diagnosis |

## See also

- [`doc/api.md`](api.md) — Rust / C++ API reference for contributors
- [`doc/development.md`](development.md) — architecture and contributor guide
- [`doc/troubleshooting.md`](troubleshooting.md) — extended troubleshooting reference
- [`samples/`](../samples/) — annotated example scripts
- [`doc/ROADMAP.md`](ROADMAP.md) — roadmap and phase status
