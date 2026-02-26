# Example Rust app runtime image with southpaw startup enforcement.

FROM debian:bookworm-slim

WORKDIR /app

# COPY --from=builder /app/target/release/my-service /app/my-service
# COPY southpaw /usr/local/bin/southpaw
COPY southpaw /usr/local/bin/southpaw
RUN chmod +x /usr/local/bin/southpaw

COPY examples/policies/rust-runtime.yaml /etc/southpaw/southpaw.yaml

ENTRYPOINT ["southpaw", "check", "--policy", "/etc/southpaw/southpaw.yaml", "--exec", "/app/my-service"]

