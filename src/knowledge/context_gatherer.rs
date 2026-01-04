//! Context gatherer for enriching process analysis with system information.
//!
//! This module collects additional context about processes to help ICA
//! provide better names and descriptions.

use std::collections::HashMap;
use std::process::Command;

use super::types::AnalysisContext;

/// Enrich an AnalysisContext with additional system information
pub fn enrich_context(ctx: &mut AnalysisContext) {
    // Get process info if we have a PID
    if let Some(pid) = ctx.pid {
        enrich_from_pid(ctx, pid);
    }

    // Get macOS app metadata if we have an executable path
    if let Some(ref path) = ctx.executable_path.clone() {
        enrich_from_macos_app(ctx, path);
    }

    // Get Docker container info if we have a container name
    if let Some(ref container) = ctx.container_name.clone() {
        enrich_from_docker(ctx, container);
    }
}

/// Gather context from process ID using ps and lsof
fn enrich_from_pid(ctx: &mut AnalysisContext, pid: u32) {
    // Get full command line
    if let Some(full_cmd) = get_process_command(pid) {
        ctx.full_command = Some(full_cmd.clone());

        // Extract executable path from full command
        if ctx.executable_path.is_none() {
            if let Some(path) = extract_executable_path(&full_cmd) {
                ctx.executable_path = Some(path);
            }
        }
    }

    // Get working directory
    if ctx.working_directory.is_none() {
        if let Some(cwd) = get_process_cwd(pid) {
            ctx.working_directory = Some(cwd);
        }
    }
}

/// Get full command line for a process
fn get_process_command(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command=", "-ww"])
        .output()
        .ok()?;

    if output.status.success() {
        let cmd = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cmd.is_empty() {
            return Some(cmd);
        }
    }
    None
}

/// Get working directory for a process using lsof
fn get_process_cwd(pid: u32) -> Option<String> {
    let output = Command::new("lsof")
        .args(["-p", &pid.to_string(), "-Fn"])
        .output()
        .ok()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // lsof -Fn outputs: p<pid>\nf<fd>\nn<name>
        // We look for "cwd" file descriptor
        let mut in_cwd = false;
        for line in output_str.lines() {
            if line == "fcwd" {
                in_cwd = true;
            } else if in_cwd && line.starts_with('n') {
                return Some(line[1..].to_string());
            } else if line.starts_with('f') {
                in_cwd = false;
            }
        }
    }
    None
}

/// Extract the executable path from a full command
fn extract_executable_path(full_cmd: &str) -> Option<String> {
    // Handle quoted paths
    if full_cmd.starts_with('"') {
        if let Some(end) = full_cmd[1..].find('"') {
            return Some(full_cmd[1..=end].to_string());
        }
    }

    // Take first space-separated token
    let path = full_cmd.split_whitespace().next()?;

    // Only return if it looks like a path
    if path.starts_with('/') || path.contains('/') {
        Some(path.to_string())
    } else {
        None
    }
}

/// Enrich context from macOS app bundle metadata
fn enrich_from_macos_app(ctx: &mut AnalysisContext, executable_path: &str) {
    // Check if this is a .app bundle
    if let Some(app_path) = extract_app_bundle_path(executable_path) {
        if let Some(metadata) = get_macos_app_metadata(&app_path) {
            ctx.macos_app_name = metadata.get("kMDItemDisplayName").cloned();
            ctx.macos_app_kind = metadata.get("kMDItemKind").cloned();
        }
    }
}

/// Extract .app bundle path from executable path
fn extract_app_bundle_path(path: &str) -> Option<String> {
    // /Applications/Foo.app/Contents/MacOS/Foo -> /Applications/Foo.app
    if let Some(pos) = path.find(".app/") {
        Some(path[..pos + 4].to_string())
    } else if path.ends_with(".app") {
        Some(path.to_string())
    } else {
        None
    }
}

/// Get macOS app metadata using mdls
fn get_macos_app_metadata(app_path: &str) -> Option<HashMap<String, String>> {
    let output = Command::new("mdls")
        .args([
            "-name",
            "kMDItemDisplayName",
            "-name",
            "kMDItemKind",
            "-name",
            "kMDItemCFBundleIdentifier",
            app_path,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut metadata = HashMap::new();

    for line in output_str.lines() {
        // Format: kMDItemDisplayName = "Control Center"
        if let Some((key, value)) = parse_mdls_line(line) {
            metadata.insert(key, value);
        }
    }

    if metadata.is_empty() {
        None
    } else {
        Some(metadata)
    }
}

/// Parse a line from mdls output
fn parse_mdls_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, " = ").collect();
    if parts.len() != 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let value = parts[1].trim();

    // Handle "(null)" values
    if value == "(null)" {
        return None;
    }

    // Remove surrounding quotes
    let value = if value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    };

    Some((key, value))
}

/// Enrich context from Docker container inspection
fn enrich_from_docker(ctx: &mut AnalysisContext, container_name: &str) {
    // Get Docker labels (compose info)
    if let Some(labels) = get_docker_labels(container_name) {
        ctx.docker_service = labels.get("com.docker.compose.service").cloned();
        ctx.docker_project = labels.get("com.docker.compose.project").cloned();

        // Get image description from OCI labels
        if let Some(desc) = labels.get("org.opencontainers.image.title") {
            ctx.docker_image = Some(desc.clone());
        } else if let Some(desc) = labels.get("org.opencontainers.image.description") {
            // Truncate long descriptions
            let truncated = if desc.len() > 100 {
                format!("{}...", &desc[..100])
            } else {
                desc.clone()
            };
            ctx.docker_image = Some(truncated);
        }
    }

    // Get Docker config (workdir, cmd)
    if let Some(config) = get_docker_config(container_name) {
        ctx.docker_workdir = config.workdir;
        ctx.docker_cmd = config.cmd;
    }
}

/// Get Docker container labels
fn get_docker_labels(container_name: &str) -> Option<HashMap<String, String>> {
    let output = Command::new("docker")
        .args(["inspect", container_name, "--format", "{{json .Config.Labels}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Parse JSON labels
    serde_json::from_str(&output_str).ok()
}

#[derive(Default)]
struct DockerConfig {
    workdir: Option<String>,
    cmd: Option<String>,
}

/// Get Docker container config (workdir, cmd)
fn get_docker_config(container_name: &str) -> Option<DockerConfig> {
    let output = Command::new("docker")
        .args([
            "inspect",
            container_name,
            "--format",
            "{{.Config.WorkingDir}}|{{.Config.Cmd}}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = output_str.splitn(2, '|').collect();

    let mut config = DockerConfig::default();

    if parts.len() >= 1 && !parts[0].is_empty() {
        config.workdir = Some(parts[0].to_string());
    }
    if parts.len() >= 2 && !parts[1].is_empty() && parts[1] != "[]" {
        // Clean up the command array format
        let cmd = parts[1]
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();
        config.cmd = Some(cmd);
    }

    Some(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_bundle_path() {
        assert_eq!(
            extract_app_bundle_path("/Applications/Safari.app/Contents/MacOS/Safari"),
            Some("/Applications/Safari.app".to_string())
        );
        assert_eq!(
            extract_app_bundle_path("/System/Library/CoreServices/ControlCenter.app/Contents/MacOS/ControlCenter"),
            Some("/System/Library/CoreServices/ControlCenter.app".to_string())
        );
        assert_eq!(extract_app_bundle_path("/usr/bin/python3"), None);
    }

    #[test]
    fn test_extract_executable_path() {
        assert_eq!(
            extract_executable_path("/usr/bin/python3 script.py"),
            Some("/usr/bin/python3".to_string())
        );
        assert_eq!(
            extract_executable_path("node server.js"),
            None // "node" alone doesn't look like a path
        );
    }

    #[test]
    fn test_parse_mdls_line() {
        assert_eq!(
            parse_mdls_line(r#"kMDItemDisplayName = "Control Center""#),
            Some(("kMDItemDisplayName".to_string(), "Control Center".to_string()))
        );
        assert_eq!(parse_mdls_line("kMDItemFoo = (null)"), None);
    }
}
