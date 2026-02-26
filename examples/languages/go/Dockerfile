# Build from repo root:
# docker build -f examples/languages/go/Dockerfile -t southpaw-go-demo .

FROM rust:1-bookworm AS southpaw-builder
WORKDIR /repo
COPY . .
RUN cargo build -p southpaw-cli --release

FROM golang:1.22-bookworm AS app-builder
WORKDIR /repo/examples/languages/go
COPY bindings/go /repo/bindings/go
COPY examples/languages/go /repo/examples/languages/go
RUN CGO_ENABLED=0 go build -o /out/go-demo .

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=southpaw-builder /repo/target/release/southpaw /usr/local/bin/southpaw
COPY --from=app-builder /out/go-demo /app/go-demo
COPY examples/languages/go/southpaw.yaml /app/southpaw.yaml
RUN useradd --uid 10001 --create-home appuser
USER appuser
ENTRYPOINT ["/app/go-demo"]

