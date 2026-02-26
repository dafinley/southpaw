# Node.js And Bun Demo

This demo uses the universal CLI startup gate:

```bash
cargo build -p southpaw-cli
PATH="$PWD/target/debug:$PATH" southpaw check --policy examples/languages/node-bun/southpaw.yaml --exec node examples/languages/node-bun/app.mjs
```

Run the same app with Bun:

```bash
cargo build -p southpaw-cli
PATH="$PWD/target/debug:$PATH" southpaw check --policy examples/languages/node-bun/southpaw.yaml --exec bun examples/languages/node-bun/app.mjs
```

Build a demo Node image from the repo root:

```bash
docker build -f examples/languages/node-bun/Dockerfile.node -t southpaw-node-demo .
docker run --rm southpaw-node-demo
```

Build a demo Bun image from the repo root:

```bash
docker build -f examples/languages/node-bun/Dockerfile.bun -t southpaw-bun-demo .
docker run --rm southpaw-bun-demo
```

The N-API wrapper in `bindings/node` is currently scaffolded. Use this CLI pattern for a working demo today.

