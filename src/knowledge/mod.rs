pub mod types;
pub mod storage;
pub mod builtin;
pub mod ica;
pub mod fallback;
pub mod learning;
pub mod worker;
pub mod context_gatherer;

// Re-export commonly used items
pub use types::{
    AnalysisContext, KnowledgeBase, KnowledgeEntry, LearningConfig, ProcessCategory,
    ProcessFingerprint,
};
pub use storage::{load_knowledge_base, save_knowledge_base};
pub use learning::{lookup_display_name, lookup_entry, record_sighting, store_result};
pub use worker::{spawn_learning_worker, AnalysisRequest, AnalysisResult};
pub use context_gatherer::enrich_context;
