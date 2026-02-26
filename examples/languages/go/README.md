# Go Demo

This demo exercises the Go wrapper module, which shells out to the `southpaw` CLI and parses JSON.

Run locally from the repo root:

```bash
cargo build -p southpaw-cli
cd examples/languages/go
PATH="$(cd ../../.. && pwd)/target/debug:$PATH" go run .
```

Build a demo image from the repo root:

```bash
docker build -f examples/languages/go/Dockerfile -t southpaw-go-demo .
docker run --rm southpaw-go-demo
```
