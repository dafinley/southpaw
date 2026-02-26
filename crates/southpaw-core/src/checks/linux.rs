#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

use crate::{
    policy::Policy,
    report::{FindingSection, FindingSeverity, PostureReport},
    Result,
};

#[derive(Debug, Default)]
struct ProcStatusSnapshot {
    uid: Option<u32>,
    euid: Option<u32>,
    gid: Option<u32>,
    egid: Option<u32>,
    seccomp: Option<u8>,
    no_new_privs: Option<bool>,
    cap_eff: Option<u64>,
}

#[derive(Debug, Clone)]
struct MountInfo {
    mount_point: String,
    mount_options: String,
    fs_type: String,
    source: String,
}

pub fn collect(policy: &Policy) -> Result<PostureReport> {
    let mut report = PostureReport::empty();

    let proc_status = match read_proc_status() {
        Ok(status) => Some(status),
        Err(err) => {
            let mut details = BTreeMap::new();
            details.insert("error".to_string(), err.to_string());
            report.add_finding_with_details(
                "runtime.proc_status_unavailable",
                FindingSection::Runtime,
                "Unable to read /proc/self/status; some checks were skipped",
                FindingSeverity::Warn,
                3,
                details,
            );
            None
        }
    };

    apply_identity(policy, proc_status.as_ref(), &mut report);
    apply_filesystem(policy, &mut report);
    apply_sandbox(policy, proc_status.as_ref(), &mut report);
    apply_network(policy, &mut report);

    report.evaluate_against(policy);
    Ok(report)
}

fn apply_identity(
    policy: &Policy,
    status: Option<&ProcStatusSnapshot>,
    report: &mut PostureReport,
) {
    let Some(status) = status else {
        return;
    };

    report.identity.uid = status.uid;
    report.identity.effective_uid = status.euid;
    report.identity.gid = status.gid;
    report.identity.effective_gid = status.egid;
    report.identity.is_root = status.euid == Some(0);
    report.identity.setuid_mismatch = status
        .uid
        .zip(status.euid)
        .map(|(a, b)| a != b)
        .unwrap_or(false);
    report.identity.setgid_mismatch = status
        .gid
        .zip(status.egid)
        .map(|(a, b)| a != b)
        .unwrap_or(false);

    if let Some(mask) = status.cap_eff {
        report.identity.effective_caps = decode_capabilities(mask);
    }

    if policy.identity.require_non_root && report.identity.is_root {
        report.add_finding(
            "identity.non_root",
            FindingSection::Identity,
            "Process is running as root (effective UID 0)",
            FindingSeverity::Fail,
            10,
        );
    }

    if !policy.identity.allowed_uids.is_empty() {
        match status.euid {
            Some(uid) if !policy.identity.allowed_uids.contains(&uid) => {
                let mut details = BTreeMap::new();
                details.insert("effective_uid".to_string(), uid.to_string());
                details.insert(
                    "allowed_uids".to_string(),
                    policy
                        .identity
                        .allowed_uids
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
                report.add_finding_with_details(
                    "identity.allowed_uids",
                    FindingSection::Identity,
                    "Effective UID is not in policy allowlist",
                    FindingSeverity::Fail,
                    9,
                    details,
                );
            }
            None => {
                report.add_finding(
                    "identity.allowed_uids_unknown",
                    FindingSection::Identity,
                    "Unable to determine effective UID for allowlist enforcement",
                    FindingSeverity::Warn,
                    3,
                );
            }
            _ => {}
        }
    }

    if report.identity.setuid_mismatch {
        report.add_finding(
            "identity.setuid_mismatch",
            FindingSection::Identity,
            "Real UID and effective UID differ (setuid or privilege transition)",
            FindingSeverity::Warn,
            4,
        );
    }

    if report.identity.setgid_mismatch {
        report.add_finding(
            "identity.setgid_mismatch",
            FindingSection::Identity,
            "Real GID and effective GID differ (setgid or privilege transition)",
            FindingSeverity::Warn,
            4,
        );
    }

    if status.cap_eff.unwrap_or(0) != 0 {
        let cap_list = if report.identity.effective_caps.is_empty() {
            "unknown".to_string()
        } else {
            report.identity.effective_caps.join(",")
        };
        let mut details = BTreeMap::new();
        details.insert("capabilities".to_string(), cap_list);
        report.add_finding_with_details(
            "identity.capabilities_present",
            FindingSection::Identity,
            "Process has effective Linux capabilities; review least-privilege settings",
            FindingSeverity::Warn,
            5,
            details,
        );
    }
}

fn apply_filesystem(policy: &Policy, report: &mut PostureReport) {
    let mountinfo = match fs::read_to_string("/proc/self/mountinfo") {
        Ok(v) => v,
        Err(err) => {
            let mut details = BTreeMap::new();
            details.insert("error".to_string(), err.to_string());
            report.add_finding_with_details(
                "filesystem.mountinfo_unavailable",
                FindingSection::Filesystem,
                "Unable to read /proc/self/mountinfo; filesystem scope checks skipped",
                FindingSeverity::Warn,
                3,
                details,
            );
            return;
        }
    };

    let mounts = parse_mountinfo(&mountinfo);
    evaluate_filesystem_mounts(policy, &mounts, report);
}

fn evaluate_filesystem_mounts(policy: &Policy, mounts: &[MountInfo], report: &mut PostureReport) {
    let suspicious_socket_markers = ["docker.sock", "containerd.sock", "podman.sock", "crio.sock"];
    let ignored_fs_types: HashSet<&str> = [
        "proc",
        "sysfs",
        "cgroup",
        "cgroup2",
        "mqueue",
        "devpts",
        "securityfs",
        "tracefs",
        "configfs",
        "pstore",
        "autofs",
        "overlay",
    ]
    .into_iter()
    .collect();

    let mut disallowed_writable = Vec::new();
    let mut suspicious_mounts = Vec::new();

    for mount in mounts {
        let mount_descriptor = format!("{} ({})", mount.mount_point, mount.source);
        if suspicious_socket_markers
            .iter()
            .any(|marker| mount.mount_point.contains(marker) || mount.source.contains(marker))
        {
            suspicious_mounts.push(mount_descriptor);
        }

        if !is_writable_mount(&mount.mount_options) {
            continue;
        }

        report
            .filesystem
            .writable_paths
            .push(mount.mount_point.clone());

        if mount.mount_point == "/" {
            report.filesystem.root_is_writable = true;
        }

        let ignored = ignored_fs_types.contains(mount.fs_type.as_str());
        if !ignored
            && !path_allowed_by_policy(
                &mount.mount_point,
                &policy.filesystem.allowed_writable_paths,
            )
        {
            disallowed_writable.push(format!("{} [{}]", mount.mount_point, mount.fs_type));
        }
    }

    report.filesystem.suspicious_mounts = suspicious_mounts.clone();

    if policy.filesystem.deny_writable_root && report.filesystem.root_is_writable {
        report.add_finding(
            "filesystem.root_writable",
            FindingSection::Filesystem,
            "Root filesystem mount (/) is writable",
            FindingSeverity::Fail,
            10,
        );
    }

    if !disallowed_writable.is_empty() {
        let mut details = BTreeMap::new();
        details.insert("paths".to_string(), disallowed_writable.join(", "));
        report.add_finding_with_details(
            "filesystem.disallowed_writable_mounts",
            FindingSection::Filesystem,
            "Writable mounts exist outside the policy allowlist",
            FindingSeverity::Warn,
            5,
            details,
        );
    }

    if !suspicious_mounts.is_empty() {
        let mut details = BTreeMap::new();
        details.insert("mounts".to_string(), suspicious_mounts.join(", "));
        report.add_finding_with_details(
            "filesystem.suspicious_host_mount",
            FindingSection::Filesystem,
            "Potential host-control socket mount detected (e.g. docker socket)",
            FindingSeverity::Fail,
            9,
            details,
        );
    }
}

fn apply_sandbox(policy: &Policy, status: Option<&ProcStatusSnapshot>, report: &mut PostureReport) {
    if let Some(status) = status {
        report.sandbox.seccomp_mode = status.seccomp;
        report.sandbox.no_new_privs = status.no_new_privs;

        if policy.sandbox.require_seccomp && status.seccomp.unwrap_or(0) == 0 {
            report.add_finding(
                "sandbox.seccomp_required",
                FindingSection::Sandbox,
                "Seccomp is not enabled for this process",
                FindingSeverity::Fail,
                10,
            );
        }

        if policy.sandbox.require_no_new_privs && status.no_new_privs != Some(true) {
            report.add_finding(
                "sandbox.no_new_privs_required",
                FindingSection::Sandbox,
                "no_new_privs is not set for this process",
                FindingSeverity::Warn,
                4,
            );
        }
    }

    report.sandbox.apparmor_enabled = read_apparmor_enabled();
    report.sandbox.apparmor_profile = read_apparmor_profile();
    report.sandbox.selinux_enforcing = read_selinux_enforcing();

    let apparmor_enforced = report
        .sandbox
        .apparmor_profile
        .as_deref()
        .map(|v| !v.is_empty() && v != "unconfined")
        .unwrap_or(false);
    let selinux_enforced = report.sandbox.selinux_enforcing == Some(true);

    if policy.sandbox.require_lsm && !(apparmor_enforced || selinux_enforced) {
        report.add_finding(
            "sandbox.lsm_required",
            FindingSection::Sandbox,
            "No enforcing AppArmor or SELinux profile detected",
            FindingSeverity::Fail,
            8,
        );
    }
}

fn apply_network(policy: &Policy, report: &mut PostureReport) {
    if !policy.network.allowed_egress_hosts.is_empty() {
        report.add_finding(
            "network.egress_validation_todo",
            FindingSection::Network,
            "allowed_egress_hosts is configured but egress validation is NOT YET IMPLEMENTED — \
             this policy is not being enforced",
            FindingSeverity::Fail,
            10,
        );
    }

    if !policy.network.block_metadata {
        return;
    }

    report.network.metadata_probe_attempted = true;

    let timeout = Duration::from_millis(policy.network.metadata_probe_timeout_ms.max(1));
    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)),
        policy.network.metadata_probe_port,
    );

    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => {
            report.network.metadata_reachable = Some(true);
            report.add_finding(
                "network.metadata_reachable",
                FindingSection::Network,
                "Cloud metadata endpoint appears reachable from the process",
                FindingSeverity::Fail,
                9,
            );
        }
        Err(_) => {
            report.network.metadata_reachable = Some(false);
        }
    }
}

fn read_proc_status() -> std::io::Result<ProcStatusSnapshot> {
    let content = fs::read_to_string("/proc/self/status")?;
    Ok(parse_proc_status(&content))
}

fn parse_proc_status(content: &str) -> ProcStatusSnapshot {
    let mut out = ProcStatusSnapshot::default();

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("Uid:") {
            let nums = parse_u32_fields(value);
            out.uid = nums.get(0).copied();
            out.euid = nums.get(1).copied();
        } else if let Some(value) = line.strip_prefix("Gid:") {
            let nums = parse_u32_fields(value);
            out.gid = nums.get(0).copied();
            out.egid = nums.get(1).copied();
        } else if let Some(value) = line.strip_prefix("Seccomp:") {
            out.seccomp = value.trim().parse::<u8>().ok();
        } else if let Some(value) = line.strip_prefix("NoNewPrivs:") {
            out.no_new_privs = match value.trim() {
                "1" => Some(true),
                "0" => Some(false),
                _ => None,
            };
        } else if let Some(value) = line.strip_prefix("CapEff:") {
            out.cap_eff = u64::from_str_radix(value.trim(), 16).ok();
        }
    }

    out
}

fn parse_u32_fields(input: &str) -> Vec<u32> {
    input
        .split_whitespace()
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

fn parse_mountinfo(input: &str) -> Vec<MountInfo> {
    input
        .lines()
        .filter_map(|line| {
            let (left, right) = line.split_once(" - ")?;
            let left_parts: Vec<&str> = left.split_whitespace().collect();
            if left_parts.len() < 6 {
                return None;
            }
            let right_parts: Vec<&str> = right.split_whitespace().collect();
            let fs_type = right_parts.get(0).copied().unwrap_or_default().to_string();
            let source = right_parts.get(1).copied().unwrap_or_default().to_string();
            Some(MountInfo {
                mount_point: decode_mount_field(left_parts[4]),
                mount_options: left_parts[5].to_string(),
                fs_type,
                source: decode_mount_field(&source),
            })
        })
        .collect()
}

fn decode_mount_field(input: &str) -> String {
    // mountinfo escapes spaces and a few characters as octal (e.g. \040)
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 3 < bytes.len() {
            let oct = &input[i + 1..i + 4];
            if let Ok(v) = u8::from_str_radix(oct, 8) {
                out.push(v as char);
                i += 4;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn is_writable_mount(mount_options: &str) -> bool {
    let has_ro = mount_options.split(',').any(|opt| opt == "ro");
    let has_rw = mount_options.split(',').any(|opt| opt == "rw");
    has_rw && !has_ro
}

fn path_allowed_by_policy(path: &str, allowed_paths: &[String]) -> bool {
    if allowed_paths.is_empty() {
        return false;
    }
    allowed_paths
        .iter()
        .any(|allowed| path_is_within(path, allowed))
}

fn path_is_within(path: &str, base: &str) -> bool {
    let norm_path = normalize_pathish(path);
    let norm_base = normalize_pathish(base);
    if norm_base == "/" {
        return true;
    }
    norm_path == norm_base
        || (norm_path.starts_with(&norm_base)
            && norm_path
                .as_bytes()
                .get(norm_base.len())
                .map(|b| *b == b'/')
                .unwrap_or(false))
}

fn normalize_pathish(path: &str) -> String {
    if path == "/" {
        return "/".to_string();
    }
    path.trim_end_matches('/').to_string()
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|v| v.trim().to_string())
}

fn read_apparmor_enabled() -> Option<bool> {
    read_trimmed("/sys/module/apparmor/parameters/enabled").map(|v| {
        let first = v.as_bytes().first().copied().unwrap_or_default();
        first == b'Y' || first == b'y'
    })
}

fn read_apparmor_profile() -> Option<String> {
    read_trimmed("/proc/self/attr/current")
}

fn read_selinux_enforcing() -> Option<bool> {
    read_trimmed("/sys/fs/selinux/enforce").map(|v| v == "1")
}

fn decode_capabilities(mask: u64) -> Vec<String> {
    const CAP_NAMES: [&str; 41] = [
        "CAP_CHOWN",
        "CAP_DAC_OVERRIDE",
        "CAP_DAC_READ_SEARCH",
        "CAP_FOWNER",
        "CAP_FSETID",
        "CAP_KILL",
        "CAP_SETGID",
        "CAP_SETUID",
        "CAP_SETPCAP",
        "CAP_LINUX_IMMUTABLE",
        "CAP_NET_BIND_SERVICE",
        "CAP_NET_BROADCAST",
        "CAP_NET_ADMIN",
        "CAP_NET_RAW",
        "CAP_IPC_LOCK",
        "CAP_IPC_OWNER",
        "CAP_SYS_MODULE",
        "CAP_SYS_RAWIO",
        "CAP_SYS_CHROOT",
        "CAP_SYS_PTRACE",
        "CAP_SYS_PACCT",
        "CAP_SYS_ADMIN",
        "CAP_SYS_BOOT",
        "CAP_SYS_NICE",
        "CAP_SYS_RESOURCE",
        "CAP_SYS_TIME",
        "CAP_SYS_TTY_CONFIG",
        "CAP_MKNOD",
        "CAP_LEASE",
        "CAP_AUDIT_WRITE",
        "CAP_AUDIT_CONTROL",
        "CAP_SETFCAP",
        "CAP_MAC_OVERRIDE",
        "CAP_MAC_ADMIN",
        "CAP_SYSLOG",
        "CAP_WAKE_ALARM",
        "CAP_BLOCK_SUSPEND",
        "CAP_AUDIT_READ",
        "CAP_PERFMON",
        "CAP_BPF",
        "CAP_CHECKPOINT_RESTORE",
    ];

    CAP_NAMES
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            if (mask & (1u64 << idx)) != 0 {
                Some((*name).to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_from_yaml(input: &str) -> Policy {
        Policy::from_str(input, Some("yaml")).expect("policy parses")
    }

    fn finding_ids(report: &PostureReport) -> Vec<&str> {
        report.findings.iter().map(|f| f.id.as_str()).collect()
    }

    #[test]
    fn parses_proc_status_identity_sandbox_and_caps() {
        let snapshot = parse_proc_status(
            r#"
Name: posture
Uid:	1000	1001	1000	1000
Gid:	2000	2001	2000	2000
CapEff:	0000000000000400
NoNewPrivs:	1
Seccomp:	2
"#,
        );

        assert_eq!(snapshot.uid, Some(1000));
        assert_eq!(snapshot.euid, Some(1001));
        assert_eq!(snapshot.gid, Some(2000));
        assert_eq!(snapshot.egid, Some(2001));
        assert_eq!(snapshot.cap_eff, Some(0x400));
        assert_eq!(snapshot.no_new_privs, Some(true));
        assert_eq!(snapshot.seccomp, Some(2));
        assert_eq!(
            decode_capabilities(snapshot.cap_eff.unwrap()),
            vec!["CAP_NET_BIND_SERVICE".to_string()]
        );
    }

    #[test]
    fn mountinfo_decodes_escaped_paths_and_writable_state() {
        let mounts = parse_mountinfo(
            "36 25 0:32 / /tmp/my\\040cache rw,relatime - tmpfs tmpfs rw,size=1024k\n",
        );

        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].mount_point, "/tmp/my cache");
        assert_eq!(mounts[0].fs_type, "tmpfs");
        assert!(is_writable_mount(&mounts[0].mount_options));
    }

    #[test]
    fn path_allowlist_respects_boundaries() {
        let allowed = vec!["/tmp".to_string()];

        assert!(path_allowed_by_policy("/tmp", &allowed));
        assert!(path_allowed_by_policy("/tmp/cache", &allowed));
        assert!(!path_allowed_by_policy("/tmpx", &allowed));
    }

    #[test]
    fn filesystem_flags_writable_root_and_disallowed_mounts() {
        let policy = policy_from_yaml(
            r#"
filesystem:
  allowed_writable_paths:
    - /tmp
  deny_writable_root: true
"#,
        );
        let mounts = parse_mountinfo(
            "1 0 8:1 / / rw,relatime - ext4 /dev/root rw\n\
             2 1 0:32 / /tmp rw,relatime - tmpfs tmpfs rw\n\
             3 1 0:33 / /var/lib/app rw,relatime - tmpfs tmpfs rw\n",
        );
        let mut report = PostureReport::empty();

        evaluate_filesystem_mounts(&policy, &mounts, &mut report);

        assert!(report.filesystem.root_is_writable);
        assert!(finding_ids(&report).contains(&"filesystem.root_writable"));
        assert!(finding_ids(&report).contains(&"filesystem.disallowed_writable_mounts"));
    }

    #[test]
    fn filesystem_detects_readonly_host_socket_mounts() {
        let policy = policy_from_yaml(
            r#"
filesystem:
  allowed_writable_paths:
    - /tmp
  deny_writable_root: true
"#,
        );
        let mounts = parse_mountinfo(
            "4 1 0:44 /docker.sock /var/run/docker.sock ro,relatime - bind /var/run/docker.sock ro\n",
        );
        let mut report = PostureReport::empty();

        evaluate_filesystem_mounts(&policy, &mounts, &mut report);

        assert!(report
            .filesystem
            .suspicious_mounts
            .iter()
            .any(|mount| mount.contains("docker.sock")));
        assert!(finding_ids(&report).contains(&"filesystem.suspicious_host_mount"));
    }

    #[test]
    fn egress_policy_is_flagged_without_metadata_probe() {
        let policy = policy_from_yaml(
            r#"
network:
  block_metadata: false
  allowed_egress_hosts:
    - api.example.com
"#,
        );
        let mut report = PostureReport::empty();

        apply_network(&policy, &mut report);

        assert!(!report.network.metadata_probe_attempted);
        assert!(finding_ids(&report).contains(&"network.egress_validation_todo"));
    }
}
