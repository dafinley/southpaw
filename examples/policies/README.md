# Ecosystem Policy Presets

These presets separate **build-stage** and **runtime-stage** posture expectations for common language stacks.

- `*-build.yaml`: relaxed checks for image build steps (caches, package installs, writable paths)
- `*-runtime.yaml`: stricter checks for production startup gating

Files:

- `python-build.yaml`, `python-runtime.yaml`
- `go-build.yaml`, `go-runtime.yaml`
- `rust-build.yaml`, `rust-runtime.yaml`

Use them as starting points, not final policy.

