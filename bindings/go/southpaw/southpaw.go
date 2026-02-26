package southpaw

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type Mode string

const (
	ModeWarn     Mode = "warn"
	ModeFail     Mode = "fail"
	ModeDegraded Mode = "degraded"
)

type Status string

const (
	StatusPass     Status = "pass"
	StatusWarn     Status = "warn"
	StatusFail     Status = "fail"
	StatusDegraded Status = "degraded"
)

type Finding struct {
	ID       string            `json:"id"`
	Section  string            `json:"section"`
	Message  string            `json:"message"`
	Severity string            `json:"severity"`
	Score    uint32            `json:"score"`
	Details  map[string]string `json:"details,omitempty"`
}

type Report struct {
	Status        Status         `json:"status"`
	RiskTotal     uint32         `json:"risk_total"`
	WarnThreshold uint32         `json:"warn_threshold"`
	FailThreshold uint32         `json:"fail_threshold"`
	Findings      []Finding      `json:"findings"`
	Identity      map[string]any `json:"identity"`
	Filesystem    map[string]any `json:"filesystem"`
	Sandbox       map[string]any `json:"sandbox"`
	Network       map[string]any `json:"network"`
}

type Options struct {
	PolicyFile string
	Mode       Mode
	Binary     string
	ExtraArgs  []string
}

type CommandError struct {
	ExitCode int
	Stdout   string
	Stderr   string
	Err      error
}

func (e *CommandError) Error() string {
	return fmt.Sprintf("southpaw command error (exit=%d): %v", e.ExitCode, e.Err)
}

func (e *CommandError) Unwrap() error {
	return e.Err
}

func Check(ctx context.Context, opts Options) (Report, error) {
	report := Report{}

	bin := opts.Binary
	if bin == "" {
		bin = "southpaw"
	}
	bin = resolveBinary(bin)

	args := []string{"check", "--format", "json"}
	if opts.PolicyFile != "" {
		args = append(args, "--policy", opts.PolicyFile)
	}
	if opts.Mode != "" {
		args = append(args, "--mode", string(opts.Mode))
	}
	if len(opts.ExtraArgs) > 0 {
		args = append(args, opts.ExtraArgs...)
	}

	cmd := exec.CommandContext(ctx, bin, args...)
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	err := cmd.Run()
	exitCode := 0
	if err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			exitCode = exitErr.ExitCode()
		} else {
			return report, &CommandError{
				ExitCode: -1,
				Stdout:   stdout.String(),
				Stderr:   stderr.String(),
				Err:      err,
			}
		}
	}

	// southpaw returns JSON for pass/warn/fail (0/1/2); treat these as successful report retrieval.
	if exitCode != 0 && exitCode != 1 && exitCode != 2 {
		return report, &CommandError{
			ExitCode: exitCode,
			Stdout:   stdout.String(),
			Stderr:   stderr.String(),
			Err:      err,
		}
	}

	if decodeErr := json.Unmarshal(stdout.Bytes(), &report); decodeErr != nil {
		return report, fmt.Errorf(
			"decode southpaw JSON output: %w (stdout=%q stderr=%q)",
			decodeErr,
			stdout.String(),
			stderr.String(),
		)
	}

	return report, nil
}

func resolveBinary(bin string) string {
	if filepath.IsAbs(bin) || strings.ContainsRune(bin, os.PathSeparator) {
		return bin
	}

	resolved, err := exec.LookPath(bin)
	if err == nil {
		return resolved
	}
	if errors.Is(err, exec.ErrDot) && resolved != "" {
		if abs, absErr := filepath.Abs(resolved); absErr == nil {
			return abs
		}
	}
	return bin
}

func Require(ctx context.Context, opts Options) (Report, error) {
	report, err := Check(ctx, opts)
	if err != nil {
		return report, err
	}
	if report.Status == StatusFail {
		return report, fmt.Errorf("southpaw failed startup checks")
	}
	return report, nil
}
