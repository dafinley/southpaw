use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

fn southpaw_bin() -> &'static str {
    env!("CARGO_BIN_EXE_southpaw")
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("southpaw-{name}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp dir created");
    dir
}

fn run(args: &[&str], cwd: &Path) -> Output {
    Command::new(southpaw_bin())
        .args(args)
        .current_dir(cwd)
        .env_remove("SOUTHPAW_POLICY")
        .env_remove("SOUTHPAW_FORMAT")
        .env_remove("SOUTHPAW_MODE")
        .output()
        .expect("southpaw command runs")
}

fn write_deterministic_fail_policy(dir: &Path) -> PathBuf {
    let path = dir.join("fail.yaml");
    fs::write(
        &path,
        r#"version: "1.0"

identity:
  require_non_root: false
  allowed_uids: []

filesystem:
  allowed_writable_paths:
    - /
  deny_writable_root: false

sandbox:
  require_seccomp: false
  require_lsm: false
  require_no_new_privs: false

network:
  block_metadata: false
  allowed_egress_hosts:
    - api.example.com

severity:
  warn_threshold: 1
  fail_threshold: 1

enforcement:
  mode: fail
"#,
    )
    .expect("policy written");
    path
}

fn stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout is valid JSON")
}

#[test]
fn help_and_version_succeed() {
    let dir = temp_dir("help-version");

    let help = run(&["--help"], &dir);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stderr).contains("southpaw CLI"));

    let version = run(&["--version"], &dir);
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("southpaw "));
}

#[test]
fn missing_policy_and_invalid_options_fail() {
    let dir = temp_dir("invalid-options");

    let missing = run(&["check", "--policy", "missing.yaml"], &dir);
    assert_eq!(missing.status.code(), Some(2));

    let bad_format = run(&["check", "--format", "xml"], &dir);
    assert_eq!(bad_format.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad_format.stderr).contains("unsupported format"));

    let bad_mode = run(&["check", "--mode", "observe"], &dir);
    assert_eq!(bad_mode.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad_mode.stderr).contains("unsupported mode"));
}

#[test]
fn json_output_and_exit_codes_cover_fail_and_warn() {
    let dir = temp_dir("json-exit");
    let policy = write_deterministic_fail_policy(&dir);

    let fail = run(
        &[
            "check",
            "--policy",
            policy.to_str().expect("utf8 path"),
            "--format",
            "json",
        ],
        &dir,
    );
    assert_eq!(fail.status.code(), Some(2));
    let fail_json = stdout_json(&fail);
    assert_eq!(fail_json["status"], "fail");
    let finding_ids: Vec<&str> = fail_json["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .filter_map(|finding| finding["id"].as_str())
        .collect();
    assert!(
        finding_ids.contains(&"network.egress_validation_todo")
            || finding_ids.contains(&"runtime.platform_unsupported")
    );

    let warn = run(
        &[
            "check",
            "--policy",
            policy.to_str().expect("utf8 path"),
            "--mode",
            "warn",
            "--format",
            "json",
        ],
        &dir,
    );
    assert_eq!(warn.status.code(), Some(1));
    let warn_json = stdout_json(&warn);
    assert_eq!(warn_json["status"], "warn");
}

#[test]
fn env_defaults_are_used_when_flags_are_absent() {
    let dir = temp_dir("env-defaults");
    let policy = write_deterministic_fail_policy(&dir);

    let output = Command::new(southpaw_bin())
        .arg("check")
        .current_dir(&dir)
        .env("SOUTHPAW_POLICY", &policy)
        .env("SOUTHPAW_FORMAT", "json")
        .env("SOUTHPAW_MODE", "warn")
        .output()
        .expect("southpaw command runs");

    assert_eq!(output.status.code(), Some(1));
    let json = stdout_json(&output);
    assert_eq!(json["status"], "warn");
}

#[test]
fn init_supports_templates_and_refuses_overwrite() {
    let dir = temp_dir("init-template");

    let created = run(&["init", "--template", "minimal"], &dir);
    assert!(created.status.success());
    let policy = fs::read_to_string(dir.join("southpaw.yaml")).expect("policy exists");
    assert!(policy.contains("require_non_root: true"));
    assert!(policy.contains("require_lsm: false"));

    let overwrite = run(&["init", "--template", "go-runtime"], &dir);
    assert_eq!(overwrite.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&overwrite.stderr).contains("already exists"));
}
