use std::{
    collections::BTreeMap,
    env,
    io::IsTerminal,
    path::PathBuf,
    process::{self, Command},
};

use southpaw_core::{
    policy::RunMode, FindingSection, FindingSeverity, Policy, PostureReport, PostureStatus,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ANSI color codes
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

struct Colors {
    red: &'static str,
    yellow: &'static str,
    green: &'static str,
    cyan: &'static str,
    bold: &'static str,
    dim: &'static str,
    reset: &'static str,
}

impl Colors {
    fn enabled() -> Self {
        Self {
            red: RED,
            yellow: YELLOW,
            green: GREEN,
            cyan: CYAN,
            bold: BOLD,
            dim: DIM,
            reset: RESET,
        }
    }

    fn disabled() -> Self {
        Self {
            red: "",
            yellow: "",
            green: "",
            cyan: "",
            bold: "",
            dim: "",
            reset: "",
        }
    }

    fn for_stdout() -> Self {
        if std::io::stdout().is_terminal() && env::var("NO_COLOR").is_err() {
            Self::enabled()
        } else {
            Self::disabled()
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug)]
struct CheckArgs {
    policy_file: PathBuf,
    output_format: OutputFormat,
    mode_override: Option<RunMode>,
    exec_cmd: Vec<String>,
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(2);
        }
    }
}

fn run() -> Result<i32, String> {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    // Handle --help / -h (can appear anywhere)
    if raw_args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return Ok(0);
    }

    // Handle --version / -v
    if raw_args.iter().any(|a| a == "-v" || a == "--version") {
        println!("southpaw {VERSION}");
        return Ok(0);
    }

    // Handle no args — show help
    if raw_args.is_empty() {
        print_help();
        return Ok(0);
    }

    // Handle `init` subcommand
    if raw_args.first().map(String::as_str) == Some("init") {
        return handle_init(&raw_args[1..]);
    }

    let args = parse_args(raw_args)?;

    let mut policy = Policy::load_from_path(&args.policy_file).map_err(|e| e.to_string())?;
    if let Some(mode) = args.mode_override {
        policy.enforcement.mode = mode;
    }
    let report = southpaw_core::check_posture(&policy).map_err(|e| e.to_string())?;

    print_report(&report, args.output_format)?;

    if !args.exec_cmd.is_empty() && !matches!(report.status, PostureStatus::Fail) {
        let cmd_display = args.exec_cmd.join(" ");
        eprintln!(
            "southpaw: {} — launching {cmd_display}",
            status_label(report.status)
        );
        return exec_process(&args.exec_cmd);
    }

    Ok(exit_code_for_status(report.status))
}

fn handle_init(args: &[String]) -> Result<i32, String> {
    let path = PathBuf::from("southpaw.yaml");
    if path.exists() {
        return Err("southpaw.yaml already exists in current directory".to_string());
    }

    let mut template_name = "default";
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => {
                i += 1;
                template_name = args.get(i).ok_or("missing value for --template")?;
            }
            value => return Err(format!("unknown init argument `{value}`")),
        }
        i += 1;
    }

    let template = policy_template(template_name)?;
    std::fs::write(&path, template).map_err(|e| format!("failed to write southpaw.yaml: {e}"))?;
    eprintln!("Created southpaw.yaml from `{template_name}` template — edit it to define your security policy.");
    Ok(0)
}

fn policy_template(name: &str) -> Result<&'static str, String> {
    match name {
        "default" => Ok(include_str!("../../../examples/southpaw.yaml")),
        "minimal" => Ok(
            r#"version: "1.0"

identity:
  require_non_root: true
  allowed_uids: []

filesystem:
  allowed_writable_paths:
    - /tmp
  deny_writable_root: true

sandbox:
  require_seccomp: false
  require_lsm: false
  require_no_new_privs: false

network:
  block_metadata: true
  metadata_probe_port: 80
  metadata_probe_timeout_ms: 200
  allowed_egress_hosts: []

severity:
  warn_threshold: 5
  fail_threshold: 10

enforcement:
  mode: fail
"#,
        ),
        "python-runtime" => Ok(include_str!("../../../examples/policies/python-runtime.yaml")),
        "go-runtime" => Ok(include_str!("../../../examples/policies/go-runtime.yaml")),
        "rust-runtime" => Ok(include_str!("../../../examples/policies/rust-runtime.yaml")),
        other => Err(format!(
            "unknown template `{other}` (expected default|minimal|python-runtime|go-runtime|rust-runtime)"
        )),
    }
}

fn parse_args(args: Vec<String>) -> Result<CheckArgs, String> {
    // Skip optional `check` subcommand (it's the only one, so it's implicit)
    let start = if args.first().map(String::as_str) == Some("check") {
        1
    } else {
        0
    };

    let mut i = start;
    let mut policy_file: Option<PathBuf> = None;
    let mut output_format: Option<OutputFormat> = None;
    let mut mode_override: Option<RunMode> = None;
    let mut exec_cmd = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
            "--policy" => {
                i += 1;
                let value = args.get(i).ok_or("missing value for --policy")?;
                policy_file = Some(PathBuf::from(value));
            }
            "--format" => {
                i += 1;
                let value = args.get(i).ok_or("missing value for --format")?;
                output_format = Some(parse_output_format(value)?);
            }
            "--json" => {
                output_format = Some(OutputFormat::Json);
            }
            "--mode" => {
                i += 1;
                let value = args.get(i).ok_or("missing value for --mode")?;
                mode_override = Some(parse_run_mode(value)?);
            }
            "--exec" => {
                i += 1;
                if i >= args.len() {
                    return Err("missing command after --exec".to_string());
                }
                exec_cmd = args[i..].to_vec();
                break;
            }
            value => return Err(format!("unknown argument `{value}`")),
        }
        i += 1;
    }

    // Resolve policy file: explicit --policy > $SOUTHPAW_POLICY > southpaw.yaml
    let policy_file = policy_file.unwrap_or_else(|| {
        env::var("SOUTHPAW_POLICY")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("southpaw.yaml"))
    });

    let output_format = match output_format {
        Some(value) => value,
        None => match env::var("SOUTHPAW_FORMAT") {
            Ok(value) => parse_output_format(&value)?,
            Err(_) => OutputFormat::Text,
        },
    };

    if mode_override.is_none() {
        if let Ok(value) = env::var("SOUTHPAW_MODE") {
            mode_override = Some(parse_run_mode(&value)?);
        }
    }

    Ok(CheckArgs {
        policy_file,
        output_format,
        mode_override,
        exec_cmd,
    })
}

fn parse_output_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err(format!("unsupported format `{value}` (expected text|json)")),
    }
}

fn parse_run_mode(value: &str) -> Result<RunMode, String> {
    match value.to_ascii_lowercase().as_str() {
        "warn" => Ok(RunMode::Warn),
        "fail" => Ok(RunMode::Fail),
        "degraded" => Ok(RunMode::Degraded),
        _ => Err(format!(
            "unsupported mode `{value}` (expected warn|fail|degraded)"
        )),
    }
}

fn exec_process(exec_cmd: &[String]) -> Result<i32, String> {
    let mut cmd = Command::new(&exec_cmd[0]);
    if exec_cmd.len() > 1 {
        cmd.args(&exec_cmd[1..]);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        Err(format!("failed to exec `{}`: {err}", exec_cmd.join(" ")))
    }

    #[cfg(not(unix))]
    {
        let status = cmd.status().map_err(|e| e.to_string())?;
        Ok(status.code().unwrap_or(1))
    }
}

fn print_help() {
    eprintln!(
        "\
southpaw CLI v{VERSION}

Runtime posture checker for Linux-hosted applications.

usage:
  southpaw [check] [OPTIONS]
  southpaw init [--template <name>]
  southpaw --version

options:
  --policy <file>     Path to policy file (default: $SOUTHPAW_POLICY or southpaw.yaml)
  --format text|json  Output format (default: text)
  --json              Shorthand for --format json
  --mode <mode>       Override enforcement mode (warn|fail|degraded)
  --exec <cmd> ...    Run <cmd> if southpaw does not fail
  -h, --help          Show this help
  -v, --version       Show version

init templates:
  default, minimal, python-runtime, go-runtime, rust-runtime

examples:
  southpaw --policy southpaw.yaml
  southpaw check --format json
  southpaw init --template python-runtime
  SOUTHPAW_MODE=warn southpaw --exec python app.py
  southpaw --exec node server.js

exit codes:
  0  pass
  1  warn or degraded
  2  fail (or error)

environment variables:
  SOUTHPAW_POLICY  Path to policy file (used when --policy is not provided)
  SOUTHPAW_FORMAT  Default output format when --format is not set (text|json)
  SOUTHPAW_MODE    Override enforcement mode (warn|fail|degraded)
  NO_COLOR        Disable colored output (any value)"
    );
}

fn print_report(report: &PostureReport, output_format: OutputFormat) -> Result<(), String> {
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(report).map_err(|e| e.to_string())?;
            println!("{json}");
        }
        OutputFormat::Text => {
            let c = Colors::for_stdout();

            let status_color = match report.status {
                PostureStatus::Pass => c.green,
                PostureStatus::Warn | PostureStatus::Degraded => c.yellow,
                PostureStatus::Fail => c.red,
            };

            println!(
                "{bold}SOUTHPAW CHECK: {sc}{status}{reset} {dim}(risk: {risk}, threshold: {thresh}){reset}",
                bold = c.bold,
                sc = status_color,
                status = status_label(report.status),
                reset = c.reset,
                dim = c.dim,
                risk = report.risk_total,
                thresh = report.fail_threshold,
            );
            println!();

            let mut by_section: BTreeMap<&str, Vec<&southpaw_core::Finding>> = BTreeMap::new();
            for finding in &report.findings {
                by_section
                    .entry(section_name(&finding.section))
                    .or_default()
                    .push(finding);
            }

            let sections = [
                (FindingSection::Identity, "Identity"),
                (FindingSection::Filesystem, "Filesystem"),
                (FindingSection::Sandbox, "Sandbox"),
                (FindingSection::Network, "Network"),
                (FindingSection::Runtime, "Runtime"),
            ];

            for (_section, name) in &sections {
                println!("{bold}{name}:{reset}", bold = c.bold, reset = c.reset);
                if let Some(findings) = by_section.get(name) {
                    for finding in findings {
                        let (marker, color) = severity_marker_colored(&finding.severity, &c);
                        println!(
                            "  {color}{marker}{reset} {msg} {dim}({id}){reset}",
                            color = color,
                            marker = marker,
                            reset = c.reset,
                            msg = finding.message,
                            dim = c.dim,
                            id = finding.id,
                        );
                    }
                } else {
                    println!("  {green}ok{reset}", green = c.green, reset = c.reset);
                }
                println!();
            }

            let risk_color = if report.risk_total >= report.fail_threshold {
                c.red
            } else if report.risk_total >= report.warn_threshold {
                c.yellow
            } else {
                c.green
            };

            println!(
                "risk: {rc}{bold}{risk}{reset} {dim}(warn: {warn}, fail: {fail}){reset}",
                rc = risk_color,
                bold = c.bold,
                risk = report.risk_total,
                reset = c.reset,
                dim = c.dim,
                warn = report.warn_threshold,
                fail = report.fail_threshold,
            );
        }
    }
    Ok(())
}

fn severity_marker_colored<'a>(sev: &FindingSeverity, c: &'a Colors) -> (&'static str, &'a str) {
    match sev {
        FindingSeverity::Info => ("i", c.cyan),
        FindingSeverity::Warn => ("!", c.yellow),
        FindingSeverity::Fail => ("x", c.red),
    }
}

fn section_name(section: &FindingSection) -> &'static str {
    match section {
        FindingSection::Identity => "Identity",
        FindingSection::Filesystem => "Filesystem",
        FindingSection::Sandbox => "Sandbox",
        FindingSection::Network => "Network",
        FindingSection::Runtime => "Runtime",
    }
}

fn status_label(status: PostureStatus) -> &'static str {
    match status {
        PostureStatus::Pass => "PASS",
        PostureStatus::Warn => "WARN",
        PostureStatus::Fail => "FAIL",
        PostureStatus::Degraded => "DEGRADED",
    }
}

fn exit_code_for_status(status: PostureStatus) -> i32 {
    match status {
        PostureStatus::Pass => 0,
        PostureStatus::Warn | PostureStatus::Degraded => 1,
        PostureStatus::Fail => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::{exit_code_for_status, policy_template};
    use southpaw_core::PostureStatus;

    #[test]
    fn exit_codes_match_documented_statuses() {
        assert_eq!(exit_code_for_status(PostureStatus::Pass), 0);
        assert_eq!(exit_code_for_status(PostureStatus::Warn), 1);
        assert_eq!(exit_code_for_status(PostureStatus::Degraded), 1);
        assert_eq!(exit_code_for_status(PostureStatus::Fail), 2);
    }

    #[test]
    fn init_templates_are_available() {
        for name in [
            "default",
            "minimal",
            "python-runtime",
            "go-runtime",
            "rust-runtime",
        ] {
            let template = policy_template(name).expect("template exists");
            assert!(template.contains("version:"));
            assert!(template.contains("enforcement:"));
        }
    }

    #[test]
    fn unknown_init_template_is_rejected() {
        let err = policy_template("ruby-runtime").expect_err("unknown template fails");
        assert!(err.contains("unknown template"));
    }
}
