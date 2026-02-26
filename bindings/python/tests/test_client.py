from __future__ import annotations

import os
import pathlib
import stat
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from southpaw import (  # noqa: E402
    PostureCommandError,
    PostureFailure,
    check_posture,
    require_posture,
)


def write_fake_southpaw(directory: pathlib.Path) -> pathlib.Path:
    path = directory / "southpaw"
    path.write_text(
        """#!/bin/sh
if [ -n "$ARGS_FILE" ]; then
  printf '%s\\n' "$*" > "$ARGS_FILE"
fi

case "$FAKE_SOUTHPAW_STATUS" in
  pass)
    printf '{"status":"pass","findings":[]}'
    exit 0
    ;;
  warn)
    printf '{"status":"warn","findings":[]}'
    exit 1
    ;;
  fail)
    printf '{"status":"fail","findings":[{"id":"x"}]}'
    exit 2
    ;;
  invalid-json)
    printf 'not-json'
    exit 0
    ;;
  command-error)
    printf 'boom' >&2
    exit 42
    ;;
esac

printf '{"status":"pass","findings":[]}'
exit 0
""",
        encoding="utf-8",
    )
    path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


class CheckPostureTests(unittest.TestCase):
    def test_fake_southpaw_pass_warn_and_fail_reports(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            southpaw_bin = write_fake_southpaw(pathlib.Path(tmp))

            for status in ["pass", "warn", "fail"]:
                report = check_posture(
                    policy_file="southpaw.yaml",
                    mode="warn",
                    southpaw_bin=str(southpaw_bin),
                    env={"FAKE_SOUTHPAW_STATUS": status},
                )
                self.assertEqual(report["status"], status)

    def test_passes_expected_cli_args(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = pathlib.Path(tmp)
            southpaw_bin = write_fake_southpaw(tmp_path)
            args_file = tmp_path / "args.txt"

            check_posture(
                policy_file="southpaw.yaml",
                mode="degraded",
                southpaw_bin=str(southpaw_bin),
                env={"ARGS_FILE": str(args_file)},
            )

            self.assertEqual(
                args_file.read_text(encoding="utf-8").strip(),
                "check --format json --policy southpaw.yaml --mode degraded",
            )

    def test_invalid_json_raises_command_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            southpaw_bin = write_fake_southpaw(pathlib.Path(tmp))

            with self.assertRaises(PostureCommandError):
                check_posture(
                    southpaw_bin=str(southpaw_bin),
                    env={"FAKE_SOUTHPAW_STATUS": "invalid-json"},
                )

    def test_non_southpaw_exit_code_raises_command_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            southpaw_bin = write_fake_southpaw(pathlib.Path(tmp))

            with self.assertRaises(PostureCommandError):
                check_posture(
                    southpaw_bin=str(southpaw_bin),
                    env={"FAKE_SOUTHPAW_STATUS": "command-error"},
                )

    def test_require_posture_only_raises_on_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            southpaw_bin = write_fake_southpaw(pathlib.Path(tmp))

            require_posture(
                southpaw_bin=str(southpaw_bin),
                env={"FAKE_SOUTHPAW_STATUS": "warn"},
            )

            with self.assertRaises(PostureFailure):
                require_posture(
                    southpaw_bin=str(southpaw_bin),
                    env={"FAKE_SOUTHPAW_STATUS": "fail"},
                )


if __name__ == "__main__":
    os.environ.setdefault("PYTHONDONTWRITEBYTECODE", "1")
    unittest.main()
