mod checks;
mod error;
pub mod policy;
pub mod report;

use std::path::Path;

pub use error::{Error, Result};
pub use policy::Policy;
pub use report::{
    Finding, FindingSection, FindingSeverity, FsReport, IdentityReport, NetworkReport,
    PostureReport, PostureStatus, SandboxReport,
};

pub fn check_posture(policy: &Policy) -> Result<PostureReport> {
    checks::collect(policy)
}

pub fn check_posture_from_path(path: impl AsRef<Path>) -> Result<PostureReport> {
    let policy = Policy::load_from_path(path)?;
    check_posture(&policy)
}

pub fn check_posture_from_policy_str(
    input: &str,
    format_hint: Option<&str>,
) -> Result<PostureReport> {
    let policy = Policy::from_str(input, format_hint)?;
    check_posture(&policy)
}

pub fn check_posture_json_from_path(path: impl AsRef<Path>) -> Result<String> {
    let report = check_posture_from_path(path)?;
    serde_json::to_string(&report).map_err(Error::Json)
}

pub fn check_posture_json_from_policy_str(
    input: &str,
    format_hint: Option<&str>,
) -> Result<String> {
    let report = check_posture_from_policy_str(input, format_hint)?;
    serde_json::to_string(&report).map_err(Error::Json)
}
