# Deno Binding Skeleton

This folder contains a Deno-first TS wrapper plus a WASM Rust crate scaffold.

Current state:

- `mod.ts` exposes the intended `checkPosture()` API
- `loader.ts` attempts to load generated WASM glue from `./pkg/...`
- Rust WASM crate (`wasm/`) exports a JSON-returning function
- End-to-end Deno collector parity is TODO (runtime probes from Deno/WASM need integration work)

Suggested future direction:

1. Build a Deno collector in TS (read `/proc`, probe metadata IP with Deno permissions)
2. Feed collected facts into Rust/WASM policy evaluator
3. Keep JSON schema aligned with `southpaw-core`

