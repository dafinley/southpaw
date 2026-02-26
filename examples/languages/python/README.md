# Python Demo

This demo exercises the Python wrapper package, which shells out to the `southpaw` CLI and parses JSON.

Run locally from the repo root:

```bash
cargo build -p southpaw-cli
python3 -m venv /tmp/southpaw-python-demo
. /tmp/southpaw-python-demo/bin/activate
pip install -e bindings/python
PATH="$PWD/target/debug:$PATH" python examples/languages/python/app.py
```

Build a demo image from the repo root:

```bash
docker build -f examples/languages/python/Dockerfile -t southpaw-python-demo .
docker run --rm southpaw-python-demo
```

