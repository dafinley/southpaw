# Rust Demo

This demo links the Rust `southpaw` crate directly into the application.

Run locally from the repo root:

```bash
cd examples/languages/rust
cargo run
```

Build a demo image from the repo root:

```bash
docker build -f examples/languages/rust/Dockerfile -t southpaw-rust-demo .
docker run --rm southpaw-rust-demo
```

Unlike the Python and Go demos, this one does not need the `southpaw` CLI at runtime because the Rust crate calls the core library directly.

