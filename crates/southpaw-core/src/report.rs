use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::policy::{Policy, RunMode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostureStatus {
    Pass,
    Warn,
    Fail,
    Degraded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Info,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingSection {
    Identity,
    Filesystem,
    Sandbox,
    Network,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub section: FindingSection,
    pub message: String,
    pub severity: FindingSeverity,
    pub score: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityReport {
    pub uid: Option<u32>,
    pub effective_uid: Option<u32>,
    pub gid: Option<u32>,
    pub effective_gid: Option<u32>,
    pub is_root: bool,
    #[serde(default)]
    pub effective_caps: Vec<String>,
    pub setuid_mismatch: bool,
    pub setgid_mismatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FsReport {
    #[serde(default)]
    pub writable_paths: Vec<String>,
    pub root_is_writable: bool,
    #[serde(default)]
    pub suspicious_mounts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SandboxReport {
    pub seccomp_mode: Option<u8>,
    pub no_new_privs: Option<bool>,
    pub apparmor_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apparmor_profile: Option<String>,
    pub selinux_enforcing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkReport {
    pub metadata_probe_attempted: bool,
    pub metadata_reachable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostureReport {
    pub status: PostureStatus,
    pub risk_total: u32,
    pub warn_threshold: u32,
    pub fail_threshold: u32,
    #[serde(default)]
    pub findings: Vec<Finding>,
    pub identity: IdentityReport,
    pub filesystem: FsReport,
    pub sandbox: SandboxReport,
    pub network: NetworkReport,
}

impl Default for PostureReport {
    fn default() -> Self {
        Self::empty()
    }
}

impl PostureReport {
    pub fn empty() -> Self {
        Self {
            status: PostureStatus::Pass,
            risk_total: 0,
            warn_threshold: 0,
            fail_threshold: 0,
            findings: Vec::new(),
            identity: IdentityReport::default(),
            filesystem: FsReport::default(),
            sandbox: SandboxReport::default(),
            network: NetworkReport::default(),
        }
    }

    pub fn add_finding(
        &mut self,
        id: impl Into<String>,
        section: FindingSection,
        message: impl Into<String>,
        severity: FindingSeverity,
        score: u32,
    ) {
        self.findings.push(Finding {
            id: id.into(),
            section,
            message: message.into(),
            severity,
            score,
            details: BTreeMap::new(),
        });
    }

    pub fn add_finding_with_details(
        &mut self,
        id: impl Into<String>,
        section: FindingSection,
        message: impl Into<String>,
        severity: FindingSeverity,
        score: u32,
        details: BTreeMap<String, String>,
    ) {
        self.findings.push(Finding {
            id: id.into(),
            section,
            message: message.into(),
            severity,
            score,
            details,
        });
    }

    pub fn evaluate_against(&mut self, policy: &Policy) {
        let total_score: u32 = self.findings.iter().map(|f| f.score).sum();
        let has_fail = self
            .findings
            .iter()
            .any(|f| matches!(f.severity, FindingSeverity::Fail));
        let has_warn = self
            .findings
            .iter()
            .any(|f| matches!(f.severity, FindingSeverity::Warn));

        self.risk_total = total_score;
        self.warn_threshold = policy.severity.warn_threshold;
        self.fail_threshold = policy.severity.fail_threshold;

        let mut status = if has_fail || total_score >= policy.severity.fail_threshold {
            PostureStatus::Fail
        } else if has_warn || total_score >= policy.severity.warn_threshold {
            PostureStatus::Warn
        } else {
            PostureStatus::Pass
        };

        status = match (policy.enforcement.mode, status) {
            (RunMode::Warn, PostureStatus::Fail) => PostureStatus::Warn,
            (RunMode::Warn, other) => other,
            (RunMode::Degraded, PostureStatus::Pass) => PostureStatus::Pass,
            (RunMode::Degraded, _) => PostureStatus::Degraded,
            (RunMode::Fail, other) => other,
        };

        self.status = status;
    }
}

#[cfg(test)]
mod tests {
    use crate::policy::{Policy, RunMode};

    use super::{FindingSection, FindingSeverity, PostureReport, PostureStatus};

    fn policy_with_mode(mode: RunMode) -> Policy {
        let mut policy = Policy::default();
        policy.enforcement.mode = mode;
        policy
    }

    #[test]
    fn evaluates_pass_when_no_findings() {
        let policy = policy_with_mode(RunMode::Fail);
        let mut report = PostureReport::empty();

        report.evaluate_against(&policy);

        assert_eq!(report.status, PostureStatus::Pass);
        assert_eq!(report.risk_total, 0);
    }

    #[test]
    fn evaluates_warn_from_warn_finding() {
        let policy = policy_with_mode(RunMode::Fail);
        let mut report = PostureReport::empty();
        report.add_finding(
            "test.warn",
            FindingSection::Runtime,
            "warn",
            FindingSeverity::Warn,
            1,
        );

        report.evaluate_against(&policy);

        assert_eq!(report.status, PostureStatus::Warn);
    }

    #[test]
    fn evaluates_fail_from_fail_finding() {
        let policy = policy_with_mode(RunMode::Fail);
        let mut report = PostureReport::empty();
        report.add_finding(
            "test.fail",
            FindingSection::Runtime,
            "fail",
            FindingSeverity::Fail,
            1,
        );

        report.evaluate_against(&policy);

        assert_eq!(report.status, PostureStatus::Fail);
    }

    #[test]
    fn warn_mode_downgrades_fail_to_warn() {
        let policy = policy_with_mode(RunMode::Warn);
        let mut report = PostureReport::empty();
        report.add_finding(
            "test.fail",
            FindingSection::Runtime,
            "fail",
            FindingSeverity::Fail,
            10,
        );

        report.evaluate_against(&policy);

        assert_eq!(report.status, PostureStatus::Warn);
    }

    #[test]
    fn degraded_mode_maps_findings_to_degraded() {
        let policy = policy_with_mode(RunMode::Degraded);
        let mut report = PostureReport::empty();
        report.add_finding(
            "test.fail",
            FindingSection::Runtime,
            "fail",
            FindingSeverity::Fail,
            10,
        );

        report.evaluate_against(&policy);

        assert_eq!(report.status, PostureStatus::Degraded);
    }
}
