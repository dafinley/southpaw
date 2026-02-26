# Southpaw

Southpaw is a runtime posture checker scaffold for Linux-hosted applications and containers.

## What is in this scaffold

- Rust workspace with:
  - `southpaw-core` (policy + report model + Linux checks skeleton)
  - `southpaw` (ergonomic Rust app wrapper)
  - `southpaw-cli` (text/JSON output, exit codes, env overrides, `--exec`)
- Node/Bun binding skeleton (`bindings/node`) with N-API Rust crate stub
- Deno binding skeleton (`bindings/deno`) with WASM crate stub and TS wrapper
- Python CLI wrapper package scaffold (`bindings/python`)
- Go CLI wrapper module scaffold (`bindings/go`)
- Example policies (`examples/southpaw.yaml`, `examples/policies/*`)
- Docker entrypoint examples for Python/Go/Rust (`examples/docker/*`)
- Runnable language demos for Node/Bun, Deno, Python, Go, and Rust (`examples/languages/*`)
- GitHub Actions CI starter workflow

## Quick start (CLI)

```bash
cargo run -p southpaw-cli -- check --policy examples/southpaw.yaml
```

JSON output:

```bash
cargo run -p southpaw-cli -- check --policy examples/southpaw.yaml --format json
```

Entrypoint mode:

```bash
cargo run -p southpaw-cli -- check --policy examples/southpaw.yaml --exec node server.js
```

Environment-driven overrides:

```bash
SOUTHPAW_POLICY=examples/policies/python-runtime.yaml SOUTHPAW_MODE=warn cargo run -p southpaw-cli -- --exec python app.py
```

Initialize a policy in the current project:

```bash
cargo run -p southpaw-cli -- init
```

## Language Support Strategy

- Universal path: Docker/entrypoint with the Rust CLI (`southpaw check --exec ...`)
- Rust apps: direct crate use via `crates/southpaw` / `crates/southpaw-core`
- Python apps: `bindings/python` wrapper (subprocess + JSON)
- Go apps: `bindings/go` wrapper (subprocess + JSON)
- Node/Bun + Deno: existing binding scaffolds

## Project layout

```text
crates/southpaw        # Ergonomic Rust wrapper crate
crates/southpaw-core   # Rust core checks + policy engine (MVP)
crates/southpaw-cli    # `southpaw check`
bindings/python              # Python wrapper package (CLI adapter)
bindings/go                  # Go wrapper module (CLI adapter)
bindings/node                # Node/Bun JS wrapper + N-API crate skeleton
bindings/deno                # Deno TS wrapper + WASM crate skeleton
examples/southpaw.yaml        # Base example policy
examples/policies            # Build/runtime presets for Python, Go, Rust
examples/docker              # Docker entrypoint patterns for Python, Go, Rust
examples/languages           # Tiny runnable demos per target language
```

## Notes

- Linux posture checks are implemented as best-effort probes against `/proc` and related system files.
- The Deno/WASM path is a scaffold only. A full Deno runtime collector bridge is still TODO.
- Python and Go bindings currently wrap the CLI (no native/CGO extension yet).
- Binding build/release pipelines are placeholders and need packaging work (publishing, prebuilds, CI matrix).
