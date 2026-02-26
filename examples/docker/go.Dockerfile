# Example Go app image with southpaw startup enforcement.

FROM gcr.io/distroless/base-debian12

WORKDIR /app

# COPY --from=builder /out/app /app/server
# COPY southpaw /usr/local/bin/southpaw
COPY southpaw /usr/local/bin/southpaw
COPY examples/policies/go-runtime.yaml /etc/southpaw/southpaw.yaml

ENTRYPOINT ["southpaw", "check", "--policy", "/etc/southpaw/southpaw.yaml", "--exec", "/app/server"]

