package southpaw

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func writeFakeSouthpaw(t *testing.T, body string) string {
	t.Helper()

	path := filepath.Join(t.TempDir(), "southpaw")
	if err := os.WriteFile(path, []byte(body), 0o755); err != nil {
		t.Fatalf("write fake southpaw: %v", err)
	}
	return path
}

func fakeSouthpawScript() string {
	return `#!/bin/sh
if [ -n "$ARGS_FILE" ]; then
  printf '%s\n' "$*" > "$ARGS_FILE"
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
`
}

func TestCheckParsesPassWarnAndFailReports(t *testing.T) {
	bin := writeFakeSouthpaw(t, fakeSouthpawScript())

	for _, status := range []Status{StatusPass, StatusWarn, StatusFail} {
		t.Run(string(status), func(t *testing.T) {
			t.Setenv("FAKE_SOUTHPAW_STATUS", string(status))

			report, err := Check(context.Background(), Options{
				Binary:     bin,
				PolicyFile: "southpaw.yaml",
				Mode:       ModeWarn,
			})
			if err != nil {
				t.Fatalf("Check returned error: %v", err)
			}
			if report.Status != status {
				t.Fatalf("status = %q, want %q", report.Status, status)
			}
		})
	}
}

func TestCheckPassesExpectedCLIArgs(t *testing.T) {
	bin := writeFakeSouthpaw(t, fakeSouthpawScript())
	argsFile := filepath.Join(t.TempDir(), "args.txt")
	t.Setenv("ARGS_FILE", argsFile)

	_, err := Check(context.Background(), Options{
		Binary:     bin,
		PolicyFile: "southpaw.yaml",
		Mode:       ModeDegraded,
	})
	if err != nil {
		t.Fatalf("Check returned error: %v", err)
	}

	args, err := os.ReadFile(argsFile)
	if err != nil {
		t.Fatalf("read args file: %v", err)
	}
	got := strings.TrimSpace(string(args))
	want := "check --format json --policy southpaw.yaml --mode degraded"
	if got != want {
		t.Fatalf("args = %q, want %q", got, want)
	}
}

func TestCheckHandlesRelativePathEntries(t *testing.T) {
	dir := t.TempDir()
	binDir := filepath.Join(dir, "bin")
	if err := os.Mkdir(binDir, 0o755); err != nil {
		t.Fatalf("create bin dir: %v", err)
	}
	if err := os.WriteFile(filepath.Join(binDir, "southpaw"), []byte(fakeSouthpawScript()), 0o755); err != nil {
		t.Fatalf("write fake southpaw: %v", err)
	}

	oldWd, err := os.Getwd()
	if err != nil {
		t.Fatalf("get cwd: %v", err)
	}
	if err := os.Chdir(dir); err != nil {
		t.Fatalf("chdir: %v", err)
	}
	t.Cleanup(func() {
		if err := os.Chdir(oldWd); err != nil {
			t.Fatalf("restore cwd: %v", err)
		}
	})

	t.Setenv("PATH", "bin"+string(os.PathListSeparator)+os.Getenv("PATH"))
	t.Setenv("FAKE_SOUTHPAW_STATUS", "pass")

	report, err := Check(context.Background(), Options{})
	if err != nil {
		t.Fatalf("Check returned error: %v", err)
	}
	if report.Status != StatusPass {
		t.Fatalf("status = %q, want %q", report.Status, StatusPass)
	}
}

func TestCheckRejectsInvalidJSON(t *testing.T) {
	bin := writeFakeSouthpaw(t, fakeSouthpawScript())
	t.Setenv("FAKE_SOUTHPAW_STATUS", "invalid-json")

	_, err := Check(context.Background(), Options{Binary: bin})
	if err == nil {
		t.Fatal("expected invalid JSON error")
	}
	if !strings.Contains(err.Error(), "decode southpaw JSON output") {
		t.Fatalf("error = %q, want decode context", err)
	}
}

func TestCheckReturnsCommandErrorForUnexpectedExitCode(t *testing.T) {
	bin := writeFakeSouthpaw(t, fakeSouthpawScript())
	t.Setenv("FAKE_SOUTHPAW_STATUS", "command-error")

	_, err := Check(context.Background(), Options{Binary: bin})
	var commandErr *CommandError
	if !errors.As(err, &commandErr) {
		t.Fatalf("error = %T, want *CommandError", err)
	}
	if commandErr.ExitCode != 42 {
		t.Fatalf("exit code = %d, want 42", commandErr.ExitCode)
	}
}

func TestRequireFailsOnFailStatusOnly(t *testing.T) {
	bin := writeFakeSouthpaw(t, fakeSouthpawScript())

	t.Setenv("FAKE_SOUTHPAW_STATUS", "warn")
	if _, err := Require(context.Background(), Options{Binary: bin}); err != nil {
		t.Fatalf("warn should not fail Require: %v", err)
	}

	t.Setenv("FAKE_SOUTHPAW_STATUS", "fail")
	if _, err := Require(context.Background(), Options{Binary: bin}); err == nil {
		t.Fatal("fail status should fail Require")
	}
}

func TestCheckHonorsContextCancellation(t *testing.T) {
	bin := writeFakeSouthpaw(t, "#!/bin/sh\nsleep 2\n")
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)
	defer cancel()

	_, err := Check(ctx, Options{Binary: bin})
	if err == nil {
		t.Fatal("expected context cancellation error")
	}
}
