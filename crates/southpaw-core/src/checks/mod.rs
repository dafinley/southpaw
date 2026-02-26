use crate::{policy::Policy, report::PostureReport, Result};

mod linux;

#[cfg(target_os = "linux")]
pub fn collect(policy: &Policy) -> Result<PostureReport> {
    linux::collect(policy)
}

#[cfg(not(target_os = "linux"))]
pub fn collect(policy: &Policy) -> Result<PostureReport> {
    use crate::report::{FindingSection, FindingSeverity};

    let mut report = PostureReport::empty();
    report.add_finding(
        "runtime.platform_unsupported",
        FindingSection::Runtime,
        "Collector scaffold only supports Linux posture checks in this build",
        FindingSeverity::Warn,
        2,
    );
    report.evaluate_against(policy);
    Ok(report)
}
