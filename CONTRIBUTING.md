# Contributing

southpaw is intentionally split into a small Rust core, a CLI, and thin language wrappers. Keep the Rust core as the source of truth for policy and report behavior.

## Setup

Required tools:

- Rust stable with `rustfmt`
- Go 1.22+
- Python 3.9+

Useful commands:

```bash
make fmt
make check
make test
```

If Cargo needs to populate its registry cache, run the same command without `--offline` once, then retry the offline command.

## Test Expectations

Before sending a change, run the narrowest relevant command and prefer `make test` for behavior changes:

```bash
make test-rust
make test-go
make test-python
make smoke-cli
```

Rust collector tests should use fixtures and pure helpers. Do not depend on the contributor machine's actual `/proc`, seccomp, AppArmor, SELinux, or container runtime posture for deterministic tests.

Python and Go wrapper tests should use fake `southpaw` executables that print JSON and return the documented exit codes:

- `0`: pass
- `1`: warn or degraded
- `2`: fail

## Policy Fixtures

When adding policy fixtures, keep them small and explicit:

- Set unrelated checks to non-enforcing values.
- Use `allowed_egress_hosts` when a deterministic failure is needed.
- Use named runtime templates from `examples/policies/` for docs and examples, not for low-level unit tests.

## Public Interfaces

Treat these as compatibility surfaces:

- CLI flags and environment variables in `southpaw check`
- JSON report fields in `PostureReport`
- Finding IDs such as `identity.non_root` and `network.egress_validation_todo`
- Python `check_posture` / `require_posture`
- Go `Check` / `Require`

Prefer additive changes and include tests for any changed exit code, finding ID, or JSON field.
