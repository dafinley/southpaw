# Python Binding (CLI Wrapper)

This package provides a Python-first API that shells out to the `southpaw` CLI and parses JSON output.

It is the fastest path to Python support because it avoids native extension packaging while preserving the Rust core as the source of truth.

## Install (local repo dev)

```bash
pip install -e ./bindings/python
```

The `southpaw` binary must be installed and available on `PATH` (or passed explicitly).

## Usage

```python
from southpaw import check_posture, require_posture

report = check_posture(policy_file="southpaw.yaml")
print(report["status"])

require_posture(policy_file="southpaw.yaml")  # raises if status == "fail"
```

## FastAPI startup example

```python
from fastapi import FastAPI
from southpaw import require_posture

app = FastAPI()

@app.on_event("startup")
def startup() -> None:
    require_posture(policy_file="southpaw.yaml")
```

