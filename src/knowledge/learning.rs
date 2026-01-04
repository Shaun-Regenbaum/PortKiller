use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{
    AnalysisContext, KnowledgeBase, KnowledgeEntry, KnowledgeSource, LearningConfig, PendingEntry,
    ProcessFingerprint,
};

/// Record a process sighting and queue for analysis if needed
pub fn record_sighting(
    kb: &mut KnowledgeBase,
    fingerprint: ProcessFingerprint,
    context: AnalysisContext,
    config: &LearningConfig,
) -> Option<AnalysisContext> {
    let hash = fingerprint.hash_key();
    let now = now_timestamp();

    // If already known, just update sightings
    if let Some(entry) = kb.entries.get_mut(&hash) {
        entry.sightings += 1;
        return None;
    }

    // Check pending list
    if let Some(pending) = kb.pending_analysis.get_mut(&hash) {
        pending.sightings += 1;
        pending.last_seen = now;

        // If reached threshold, return context for analysis
        if pending.sightings >= config.min_sightings {
            return Some(pending.context.clone());
        }

        return None;
    }

    // New process - add to pending if room
    if kb.pending_analysis.len() < config.max_pending {
        kb.pending_analysis.insert(
            hash,
            PendingEntry {
                fingerprint,
                sightings: 1,
                first_seen: now,
                last_seen: now,
                context,
            },
        );
    }

    None
}

/// Store analysis result in the knowledge base
pub fn store_result(
    kb: &mut KnowledgeBase,
    fingerprint: ProcessFingerprint,
    response: super::types::IcaAnalysisResponse,
    source: KnowledgeSource,
) {
    let hash = fingerprint.hash_key();
    let now = now_timestamp();

    // Remove from pending
    let sightings = kb
        .pending_analysis
        .remove(&hash)
        .map(|p| p.sightings)
        .unwrap_or(1);

    // Create entry
    let entry = KnowledgeEntry {
        fingerprint,
        display_name: response.display_name,
        description: response.description,
        category: response.category,
        group_id: response.group_hint,
        confidence: response.confidence,
        source,
        sightings,
        updated_at: now,
    };

    kb.entries.insert(hash, entry);
}

/// Look up a display name for a process
pub fn lookup_display_name(kb: &KnowledgeBase, fingerprint: &ProcessFingerprint) -> Option<String> {
    let hash = fingerprint.hash_key();
    kb.entries.get(&hash).map(|e| e.display_name.clone())
}

/// Look up full entry for a process
pub fn lookup_entry<'a>(kb: &'a KnowledgeBase, fingerprint: &ProcessFingerprint) -> Option<&'a KnowledgeEntry> {
    let hash = fingerprint.hash_key();
    kb.entries.get(&hash)
}

/// Clean up old pending entries (entries that haven't been seen recently)
pub fn cleanup_stale_pending(kb: &mut KnowledgeBase, max_age_secs: i64) {
    let now = now_timestamp();
    let cutoff = now - max_age_secs;

    kb.pending_analysis
        .retain(|_, entry| entry.last_seen > cutoff);
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> LearningConfig {
        LearningConfig {
            enabled: true,
            min_sightings: 2,
            rate_limit_secs: 5,
            max_pending: 10,
            ica_url: "http://localhost:4000".to_string(),
            setec_url: "https://setec.tailb726.ts.net".to_string(),
        }
    }

    #[test]
    fn test_first_sighting_adds_to_pending() {
        let mut kb = KnowledgeBase::default();
        let config = test_config();
        let fp = ProcessFingerprint::new("node");
        let ctx = AnalysisContext {
            command: "node".to_string(),
            port: Some(3000),
            project_name: None,
            container_name: None,
            container_prefix: None,
        };

        let result = record_sighting(&mut kb, fp.clone(), ctx, &config);

        assert!(result.is_none());
        assert!(kb.pending_analysis.contains_key(&fp.hash_key()));
    }

    #[test]
    fn test_second_sighting_returns_context() {
        let mut kb = KnowledgeBase::default();
        let config = test_config();
        let fp = ProcessFingerprint::new("node");
        let ctx = AnalysisContext {
            command: "node".to_string(),
            port: Some(3000),
            project_name: None,
            container_name: None,
            container_prefix: None,
        };

        // First sighting
        record_sighting(&mut kb, fp.clone(), ctx.clone(), &config);

        // Second sighting should return context for analysis
        let result = record_sighting(&mut kb, fp.clone(), ctx, &config);
        assert!(result.is_some());
    }

    #[test]
    fn test_known_process_not_queued() {
        let mut kb = KnowledgeBase::default();
        let config = test_config();
        let fp = ProcessFingerprint::new("node");

        // Add known entry
        kb.entries.insert(
            fp.hash_key(),
            KnowledgeEntry {
                fingerprint: fp.clone(),
                display_name: "Node.js".to_string(),
                description: "Test".to_string(),
                category: super::super::types::ProcessCategory::Backend,
                group_id: None,
                confidence: 1.0,
                source: KnowledgeSource::Builtin,
                sightings: 5,
                updated_at: 0,
            },
        );

        let ctx = AnalysisContext {
            command: "node".to_string(),
            port: Some(3000),
            project_name: None,
            container_name: None,
            container_prefix: None,
        };

        let result = record_sighting(&mut kb, fp.clone(), ctx, &config);
        assert!(result.is_none());
        assert!(!kb.pending_analysis.contains_key(&fp.hash_key()));
    }
}
