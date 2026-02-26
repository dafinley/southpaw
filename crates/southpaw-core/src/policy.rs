use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Policy {
    pub version: String,
    pub identity: IdentityPolicy,
    pub filesystem: FilesystemPolicy,
    pub sandbox: SandboxPolicy,
    pub network: NetworkPolicy,
    pub severity: SeverityPolicy,
    pub enforcement: EnforcementPolicy,
}

impl Policy {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let ext_hint = path.extension().and_then(|s| s.to_str());
        Self::from_str(&content, ext_hint)
    }

    pub fn from_str(input: &str, format_hint: Option<&str>) -> Result<Self> {
        let hint = format_hint.map(|s| s.to_ascii_lowercase());
        match hint.as_deref() {
            Some("json") => serde_json::from_str(input).map_err(Error::Json),
            Some("yaml") | Some("yml") => serde_yaml::from_str(input).map_err(Error::Yaml),
            Some(other) => Err(Error::InvalidPolicyFormat(other.to_string())),
            None => {
                let trimmed = input.trim_start();
                if trimmed.starts_with('{') {
                    serde_json::from_str(input).map_err(Error::Json)
                } else {
                    serde_yaml::from_str(input).map_err(Error::Yaml)
                }
            }
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            identity: IdentityPolicy::default(),
            filesystem: FilesystemPolicy::default(),
            sandbox: SandboxPolicy::default(),
            network: NetworkPolicy::default(),
            severity: SeverityPolicy::default(),
            enforcement: EnforcementPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IdentityPolicy {
    pub require_non_root: bool,
    pub allowed_uids: Vec<u32>,
}

impl Default for IdentityPolicy {
    fn default() -> Self {
        Self {
            require_non_root: true,
            allowed_uids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilesystemPolicy {
    pub allowed_writable_paths: Vec<String>,
    pub deny_writable_root: bool,
}

impl Default for FilesystemPolicy {
    fn default() -> Self {
        Self {
            allowed_writable_paths: vec!["/tmp".to_string()],
            deny_writable_root: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SandboxPolicy {
    pub require_seccomp: bool,
    pub require_lsm: bool,
    pub require_no_new_privs: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            require_seccomp: false,
            require_lsm: false,
            require_no_new_privs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkPolicy {
    pub block_metadata: bool,
    pub metadata_probe_port: u16,
    pub metadata_probe_timeout_ms: u64,
    pub allowed_egress_hosts: Vec<String>,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            block_metadata: false,
            metadata_probe_port: 80,
            metadata_probe_timeout_ms: 200,
            allowed_egress_hosts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SeverityPolicy {
    pub warn_threshold: u32,
    pub fail_threshold: u32,
}

impl Default for SeverityPolicy {
    fn default() -> Self {
        Self {
            warn_threshold: 5,
            fail_threshold: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EnforcementPolicy {
    pub mode: RunMode,
}

impl Default for EnforcementPolicy {
    fn default() -> Self {
        Self {
            mode: RunMode::Fail,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    Warn,
    Fail,
    Degraded,
}

impl Default for RunMode {
    fn default() -> Self {
        Self::Fail
    }
}

#[cfg(test)]
mod tests {
    use super::Policy;

    #[test]
    fn parses_yaml_policy() {
        let input = r#"
version: "1.0"
identity:
  require_non_root: true
"#;
        let policy = Policy::from_str(input, Some("yaml")).expect("policy parses");
        assert!(policy.identity.require_non_root);
        assert_eq!(policy.version, "1.0");
    }

    #[test]
    fn parses_json_policy_and_applies_defaults() {
        let input = r#"{"version":"1.0","network":{"block_metadata":true}}"#;
        let policy = Policy::from_str(input, Some("json")).expect("policy parses");

        assert_eq!(policy.version, "1.0");
        assert!(policy.network.block_metadata);
        assert!(policy.identity.require_non_root);
        assert_eq!(policy.filesystem.allowed_writable_paths, vec!["/tmp"]);
        assert_eq!(policy.severity.warn_threshold, 5);
        assert_eq!(policy.severity.fail_threshold, 10);
    }
}
