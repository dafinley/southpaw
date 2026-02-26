GOCACHE ?= /tmp/southpaw-go-build
PYTHON ?= python3

.PHONY: fmt check test test-rust test-go test-python smoke-cli

fmt:
	cargo fmt --all
	gofmt -w bindings/go/southpaw/*.go

check:
	cargo check -p southpaw-core -p southpaw-cli -p southpaw --offline
	cd bindings/go && GOCACHE=$(GOCACHE) go test ./...
	PYTHONDONTWRITEBYTECODE=1 $(PYTHON) -c "import ast, pathlib; [ast.parse(pathlib.Path(p).read_text(), filename=p) for p in ('bindings/python/southpaw/__init__.py', 'bindings/python/southpaw/client.py')]"

test: test-rust test-go test-python smoke-cli

test-rust:
	cargo test -p southpaw-core -p southpaw-cli -p southpaw --offline

test-go:
	cd bindings/go && GOCACHE=$(GOCACHE) go test ./...

test-python:
	PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=bindings/python $(PYTHON) -m unittest discover -s bindings/python/tests

smoke-cli:
	cargo run -p southpaw-cli -- check --policy examples/southpaw.yaml --format json >/tmp/southpaw-smoke.json || { status=$$?; test $$status -eq 1 -o $$status -eq 2; }
	test -s /tmp/southpaw-smoke.json
