# Deno Demo

This demo uses the universal CLI startup gate:

```bash
cargo build -p southpaw-cli
PATH="$PWD/target/debug:$PATH" southpaw check --policy examples/languages/deno/southpaw.yaml --exec deno run examples/languages/deno/main.ts
```

Build a demo image from the repo root:

```bash
docker build -f examples/languages/deno/Dockerfile -t southpaw-deno-demo .
docker run --rm southpaw-deno-demo
```

The WASM wrapper in `bindings/deno` is currently scaffolded. Use this CLI pattern for a working demo today.

