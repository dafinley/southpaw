# Language Demo Examples

These examples are intentionally tiny. Each folder includes:

- a small app
- a demo posture policy
- one README with local and Docker commands
- a Dockerfile where that is the simplest reproducible demo path

## Build the CLI Once

Most demos use the `southpaw` CLI as the universal startup gate:

```bash
cargo build -p southpaw-cli
export PATH="$PWD/target/debug:$PATH"
```

## Targets

- `node-bun`: Node.js and Bun via `southpaw check --exec`
- `deno`: Deno via `southpaw check --exec`
- `python`: Python wrapper package calling the CLI
- `go`: Go wrapper module calling the CLI
- `rust`: Rust crate linked directly into the app

The Node/Bun and Deno native binding paths are still scaffolds. These demos use the CLI path because it is the stable cross-language integration.

On non-Linux machines, local demos may report `warn` because the Linux collector intentionally returns `runtime.platform_unsupported`. The Docker demos are the better approximation of the production container behavior.
