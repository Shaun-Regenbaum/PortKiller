use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::types::KnowledgeBase;

const KNOWLEDGE_FILE: &str = ".portkiller-knowledge.json";
const CURRENT_VERSION: u32 = 1;

/// Get the path to the knowledge base file
pub fn get_knowledge_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(KNOWLEDGE_FILE)
}

/// Load the knowledge base from disk, creating a new one if it doesn't exist
pub fn load_knowledge_base() -> Result<KnowledgeBase> {
    let path = get_knowledge_path();

    if path.exists() {
        let content = fs::read_to_string(&path).context("failed to read knowledge base file")?;
        let mut kb: KnowledgeBase =
            serde_json::from_str(&content).context("failed to parse knowledge base file")?;

        // Handle version migrations if needed
        if kb.version < CURRENT_VERSION {
            kb = migrate_knowledge_base(kb)?;
            save_knowledge_base(&kb)?;
        }

        Ok(kb)
    } else {
        // Create new knowledge base with builtins
        let mut kb = KnowledgeBase::default();
        kb.version = CURRENT_VERSION;
        super::builtin::populate_builtins(&mut kb);
        save_knowledge_base(&kb)?;
        Ok(kb)
    }
}

/// Save the knowledge base to disk
pub fn save_knowledge_base(kb: &KnowledgeBase) -> Result<()> {
    let path = get_knowledge_path();
    let content =
        serde_json::to_string_pretty(kb).context("failed to serialize knowledge base")?;
    fs::write(&path, &content).context("failed to write knowledge base file")?;
    // Set secure permissions (owner read/write only)
    fs::set_permissions(&path, Permissions::from_mode(0o600))
        .context("failed to set knowledge base file permissions")?;
    Ok(())
}

/// Migrate knowledge base from older versions
fn migrate_knowledge_base(mut kb: KnowledgeBase) -> Result<KnowledgeBase> {
    // Future migrations can be added here
    // For now, just update the version
    kb.version = CURRENT_VERSION;
    Ok(kb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_knowledge_path() {
        let path = get_knowledge_path();
        assert!(path.to_string_lossy().ends_with(KNOWLEDGE_FILE));
    }
}
