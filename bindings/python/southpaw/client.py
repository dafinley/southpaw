from __future__ import annotations

import json
import os
import subprocess
from typing import Any, Mapping, MutableMapping, Sequence


PostureReport = dict[str, Any]


class PostureCommandError(RuntimeError):
    pass


class PostureFailure(RuntimeError):
    def __init__(self, report: PostureReport):
        self.report = report
        super().__init__(f"southpaw failed startup checks (status={report.get('status')})")


def check_posture(
    *,
    policy_file: str | None = None,
    mode: str | None = None,
    southpaw_bin: str = "southpaw",
    extra_args: Sequence[str] | None = None,
    env: Mapping[str, str] | None = None,
) -> PostureReport:
    cmd = [southpaw_bin, "check", "--format", "json"]
    if policy_file:
        cmd.extend(["--policy", policy_file])
    if mode:
        cmd.extend(["--mode", mode])
    if extra_args:
        cmd.extend(extra_args)

    proc_env: MutableMapping[str, str] = dict(os.environ)
    if env:
        proc_env.update(env)

    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        check=False,
        env=proc_env,
    )

    # `southpaw` uses 0/1/2 for pass/warn/fail. These are expected and still return JSON.
    if result.returncode not in (0, 1, 2):
        raise PostureCommandError(
            "southpaw command failed "
            f"(exit={result.returncode}): {result.stderr.strip() or result.stdout.strip()}"
        )

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise PostureCommandError(
            "southpaw command did not produce valid JSON output. "
            f"stdout={result.stdout!r} stderr={result.stderr!r}"
        ) from exc


def require_posture(**kwargs: Any) -> PostureReport:
    report = check_posture(**kwargs)
    if report.get("status") == "fail":
        raise PostureFailure(report)
    return report
