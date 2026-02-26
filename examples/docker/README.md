# Docker Examples

These Dockerfiles demonstrate the startup-gate pattern:

1. Copy the `southpaw` CLI binary into the runtime image
2. Copy a runtime policy
3. Use `ENTRYPOINT ["southpaw", "check", ..., "--exec", "<app>"]`

Notes:

- The `COPY southpaw ...` lines assume you provide a prebuilt binary (for example from CI artifacts).
- On Unix, `southpaw --exec ...` uses `exec`, so your app becomes PID 1 after the check.
- Use the `examples/policies/*-build.yaml` files only during build-stage validation; runtime should use the stricter `*-runtime.yaml` presets.
- For complete tiny apps with local and Docker commands, see `examples/languages/`.
