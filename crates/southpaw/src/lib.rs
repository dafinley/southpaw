use std::{path::Path, process};

pub use southpaw_core::{
    check_posture, check_posture_from_path, check_posture_from_policy_str, policy, report, Error,
    Finding, FindingSection, FindingSeverity, FsReport, IdentityReport, NetworkReport, Policy,
    PostureReport, PostureStatus, Result, SandboxReport,
};

#[derive(Debug)]
pub enum StartupError {
    Core(Error),
    Failed(PostureReport),
}

impl std::fmt::Display for StartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(err) => write!(f, "{err}"),
            Self::Failed(report) => {
                write!(
                    f,
                    "southpaw failed startup checks (status={})",
                    report_status(report)
                )
            }
        }
    }
}

impl std::error::Error for StartupError {}

impl From<Error> for StartupError {
    fn from(value: Error) -> Self {
        Self::Core(value)
    }
}

pub fn check_with_policy_path(path: impl AsRef<Path>) -> Result<PostureReport> {
    check_posture_from_path(path)
}

pub fn require_startup_posture(
    path: impl AsRef<Path>,
) -> std::result::Result<PostureReport, StartupError> {
    let report = check_posture_from_path(path).map_err(StartupError::Core)?;
    if matches!(report.status, PostureStatus::Fail) {
        return Err(StartupError::Failed(report));
    }
    Ok(report)
}

pub fn check_or_exit(path: impl AsRef<Path>) -> PostureReport {
    match check_posture_from_path(path) {
        Ok(report) => {
            if matches!(report.status, PostureStatus::Fail) {
                eprintln!("southpaw: startup blocked (status=fail)");
                process::exit(2);
            }
            report
        }
        Err(err) => {
            eprintln!("southpaw: posture check error: {err}");
            process::exit(2);
        }
    }
}

fn report_status(report: &PostureReport) -> &'static str {
    match report.status {
        PostureStatus::Pass => "pass",
        PostureStatus::Warn => "warn",
        PostureStatus::Fail => "fail",
        PostureStatus::Degraded => "degraded",
    }
}
