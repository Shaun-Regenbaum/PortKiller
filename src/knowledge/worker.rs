use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};

use super::fallback::generate_fallback;
use super::ica::IcaClient;
use super::types::{AnalysisContext, IcaAnalysisResponse, KnowledgeSource, LearningConfig, ProcessFingerprint};

/// Message sent to the learning worker
#[derive(Debug)]
pub struct AnalysisRequest {
    pub fingerprint: ProcessFingerprint,
    pub context: AnalysisContext,
}

/// Message sent back from the worker
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub fingerprint: ProcessFingerprint,
    pub response: IcaAnalysisResponse,
    pub source: KnowledgeSource,
}

/// User event for knowledge updates
#[derive(Debug, Clone)]
pub enum KnowledgeEvent {
    AnalysisComplete(AnalysisResult),
    SaveKnowledgeBase,
}

/// Spawn the background learning worker
pub fn spawn_learning_worker(
    config: Arc<LearningConfig>,
    rx: Receiver<AnalysisRequest>,
    result_tx: Sender<AnalysisResult>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let client = IcaClient::new(&config);
        let rate_limit = Duration::from_secs(config.rate_limit_secs);
        let mut last_call = Instant::now() - rate_limit; // Allow immediate first call

        log::info!(
            "Learning worker started (ICA available: {})",
            client.is_available()
        );

        for request in rx {
            // Rate limiting
            let elapsed = last_call.elapsed();
            if elapsed < rate_limit {
                thread::sleep(rate_limit - elapsed);
            }
            last_call = Instant::now();

            log::debug!(
                "Analyzing process: {} (port: {:?})",
                request.context.command,
                request.context.port
            );

            // Try ICA first, fall back to heuristics
            let (response, source) = if client.is_available() {
                match client.analyze(&request.context) {
                    Ok(resp) => {
                        log::info!(
                            "ICA analysis successful: {} -> {}",
                            request.context.command,
                            resp.display_name
                        );
                        (resp, KnowledgeSource::ApiLearned)
                    }
                    Err(e) => {
                        log::warn!(
                            "ICA analysis failed for {}: {}, using fallback",
                            request.context.command,
                            e
                        );
                        (generate_fallback(&request.context), KnowledgeSource::Heuristic)
                    }
                }
            } else {
                log::debug!(
                    "ICA not available, using heuristics for {}",
                    request.context.command
                );
                (generate_fallback(&request.context), KnowledgeSource::Heuristic)
            };

            // Send result back
            let result = AnalysisResult {
                fingerprint: request.fingerprint,
                response,
                source,
            };

            if let Err(e) = result_tx.send(result) {
                log::error!("Failed to send analysis result: {}", e);
            }
        }

        log::info!("Learning worker shutting down");
    })
}
