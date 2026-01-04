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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub command: String,
    pub port: Option<u16>,
    pub project_name: Option<String>,
    pub container_name: Option<String>,
    pub container_prefix: Option<String>,
}

impl AnalysisContext {
    pub fn to_prompt(&self) -> String {
        let mut lines = vec![];
        lines.push(format!("Command: {}", self.command));
        if let Some(port) = self.port {
            lines.push(format!("Port: {}", port));
        }
        if let Some(ref project) = self.project_name {
            lines.push(format!("Project: {}", project));
        }
        if let Some(ref container) = self.container_name {
            lines.push(format!("Docker container: {}", container));
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
