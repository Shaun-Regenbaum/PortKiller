use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Unique identifier for a process based on its characteristics
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ProcessFingerprint {
    /// The command name (e.g., "node", "python", "ruby")
    pub command: String,
    /// Default port if consistently observed
    pub default_port: Option<u16>,
    /// Hash of project directory for project-specific entries
    pub project_hash: Option<String>,
    /// Docker container prefix (e.g., "dss" from "dss_app")
    pub container_prefix: Option<String>,
}

impl ProcessFingerprint {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            default_port: None,
            project_hash: None,
            container_prefix: None,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.default_port = Some(port);
        self
    }

    pub fn with_project_hash(mut self, hash: &str) -> Self {
        self.project_hash = Some(hash.to_string());
        self
    }

    pub fn with_container_prefix(mut self, prefix: &str) -> Self {
        self.container_prefix = Some(prefix.to_string());
        self
    }

    /// Generate a unique hash key for lookups
    pub fn hash_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.command.hash(&mut hasher);
        self.default_port.hash(&mut hasher);
        self.project_hash.hash(&mut hasher);
        self.container_prefix.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// Category of process for grouping and display
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessCategory {
    Frontend,
    Backend,
    Database,
    Cache,
    Proxy,
    DevTool,
    Infrastructure,
    Unknown,
}

impl Default for ProcessCategory {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Source of knowledge entry
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeSource {
    /// Hardcoded in the application
    Builtin,
    /// Learned from ICA API
    ApiLearned,
    /// Generated from heuristics (command name, project, etc.)
    Heuristic,
}

impl Default for KnowledgeSource {
    fn default() -> Self {
        Self::Heuristic
    }
}

/// A learned piece of knowledge about a process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Fingerprint that identifies this process
    pub fingerprint: ProcessFingerprint,
    /// Human-friendly display name (e.g., "DSS Backend API")
    pub display_name: String,
    /// Description of what this process does
    pub description: String,
    /// Category for grouping
    pub category: ProcessCategory,
    /// Optional group identifier for related services
    pub group_id: Option<String>,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
    /// How this knowledge was obtained
    pub source: KnowledgeSource,
    /// Number of times this process has been seen
    pub sightings: u32,
    /// Unix timestamp of last update
    pub updated_at: i64,
}

impl KnowledgeEntry {
    pub fn hash_key(&self) -> String {
        self.fingerprint.hash_key()
    }
}

/// The persistent knowledge base
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Version for migration purposes
    pub version: u32,
    /// Entries indexed by fingerprint hash
    pub entries: HashMap<String, KnowledgeEntry>,
    /// Pending analysis queue (fingerprint hashes -> sighting count)
    #[serde(default)]
    pub pending_analysis: HashMap<String, PendingEntry>,
}

/// Entry waiting to be analyzed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingEntry {
    pub fingerprint: ProcessFingerprint,
    pub sightings: u32,
    pub first_seen: i64,
    pub last_seen: i64,
    /// Context for analysis
    pub context: AnalysisContext,
}

/// Context passed to ICA for analysis
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// The command/process name
    pub command: String,
    /// Port the process is listening on
    pub port: Option<u16>,
    /// Project directory name
    pub project_name: Option<String>,
    /// Docker container name (if containerized)
    pub container_name: Option<String>,
    /// Docker container prefix (e.g., "dss" from "dss_app")
    pub container_prefix: Option<String>,

    // === Enhanced context fields ===

    /// Full executable path (e.g., "/Applications/Foo.app/Contents/MacOS/Foo")
    pub executable_path: Option<String>,
    /// Working directory of the process
    pub working_directory: Option<String>,
    /// Full command line with arguments
    pub full_command: Option<String>,
    /// macOS app display name from mdls (e.g., "Control Center")
    pub macos_app_name: Option<String>,
    /// macOS app kind from mdls (e.g., "Application")
    pub macos_app_kind: Option<String>,
    /// Docker compose service name
    pub docker_service: Option<String>,
    /// Docker compose project name
    pub docker_project: Option<String>,
    /// Docker image name/description
    pub docker_image: Option<String>,
    /// Docker container working directory
    pub docker_workdir: Option<String>,
    /// Docker container command
    pub docker_cmd: Option<String>,
    /// Process ID (for additional lookups)
    pub pid: Option<u32>,
}

impl AnalysisContext {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            ..Default::default()
        }
    }

    pub fn to_prompt(&self) -> String {
        let mut lines = vec![];
        lines.push(format!("Command: {}", self.command));

        if let Some(port) = self.port {
            lines.push(format!("Port: {}", port));
        }
        if let Some(ref path) = self.executable_path {
            lines.push(format!("Executable: {}", path));
        }
        if let Some(ref full_cmd) = self.full_command {
            // Truncate very long commands
            let truncated = if full_cmd.len() > 200 {
                format!("{}...", &full_cmd[..200])
            } else {
                full_cmd.clone()
            };
            lines.push(format!("Full command: {}", truncated));
        }
        if let Some(ref cwd) = self.working_directory {
            lines.push(format!("Working directory: {}", cwd));
        }
        if let Some(ref project) = self.project_name {
            lines.push(format!("Project: {}", project));
        }

        // macOS app info
        if let Some(ref app_name) = self.macos_app_name {
            lines.push(format!("macOS App Name: {}", app_name));
        }
        if let Some(ref app_kind) = self.macos_app_kind {
            lines.push(format!("macOS App Kind: {}", app_kind));
        }

        // Docker info
        if let Some(ref container) = self.container_name {
            lines.push(format!("Docker container: {}", container));
        }
        if let Some(ref service) = self.docker_service {
            lines.push(format!("Docker compose service: {}", service));
        }
        if let Some(ref project) = self.docker_project {
            lines.push(format!("Docker compose project: {}", project));
        }
        if let Some(ref image) = self.docker_image {
            lines.push(format!("Docker image: {}", image));
        }
        if let Some(ref workdir) = self.docker_workdir {
            lines.push(format!("Container workdir: {}", workdir));
        }
        if let Some(ref cmd) = self.docker_cmd {
            lines.push(format!("Container command: {}", cmd));
        }
        if let Some(ref prefix) = self.container_prefix {
            lines.push(format!("Container prefix: {}", prefix));
        }

        lines.join("\n")
    }
}

/// Response from ICA analysis
#[derive(Clone, Debug, serde::Serialize, Deserialize)]
pub struct IcaAnalysisResponse {
    pub display_name: String,
    pub description: String,
    pub category: ProcessCategory,
    pub group_hint: Option<String>,
    pub confidence: f32,
}

/// Learning configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LearningConfig {
    /// Enable AI-powered learning
    pub enabled: bool,
    /// Minimum sightings before analysis
    pub min_sightings: u32,
    /// Rate limit in seconds between API calls
    pub rate_limit_secs: u64,
    /// Maximum pending entries
    pub max_pending: usize,
    /// ICA server URL
    pub ica_url: String,
    /// Setec server URL for retrieving service key
    pub setec_url: String,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_sightings: 2,
            rate_limit_secs: 5,
            max_pending: 20,
            ica_url: "https://ica.tailb726.ts.net".to_string(),
            setec_url: "https://setec.tailb726.ts.net".to_string(),
        }
    }
}
