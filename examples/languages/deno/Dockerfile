# Build from repo root:
# docker build -f examples/languages/deno/Dockerfile -t southpaw-deno-demo .

FROM rust:1-bookworm AS southpaw-builder
WORKDIR /repo
COPY . .
RUN cargo build -p southpaw-cli --release

FROM denoland/deno:debian
WORKDIR /app
COPY --from=southpaw-builder /repo/target/release/southpaw /usr/local/bin/southpaw
COPY examples/languages/deno/main.ts /app/main.ts
COPY examples/languages/deno/southpaw.yaml /app/southpaw.yaml
USER deno
ENTRYPOINT ["southpaw", "check", "--policy", "/app/southpaw.yaml", "--exec", "deno", "run", "/app/main.ts"]

