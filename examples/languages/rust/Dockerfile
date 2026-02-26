# Build from repo root:
# docker build -f examples/languages/rust/Dockerfile -t southpaw-rust-demo .

FROM rust:1-bookworm AS app-builder
WORKDIR /repo
COPY . .
WORKDIR /repo/examples/languages/rust
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=app-builder /repo/examples/languages/rust/target/release/southpaw-rust-demo /app/rust-demo
COPY examples/languages/rust/southpaw.yaml /app/southpaw.yaml
RUN useradd --uid 10001 --create-home appuser
USER appuser
ENTRYPOINT ["/app/rust-demo"]

