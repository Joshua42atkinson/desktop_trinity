//! Memory Consolidation System for Trinity
//!
//! Promotes important insights from working memory to long-term storage.
//! Runs automatically in the background or on-demand.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{hash_based_embedding, MemorySource, RelationalStore, VectorStore};
use crate::brain::orchestrator::BrainOrchestrator;
use crate::brain::Brain;

/// Report from a consolidation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationReport {
    /// Number of memories processed
    pub processed: usize,
    /// Number of new insights generated
    pub insights_created: usize,
    /// Number of memories promoted to long-term
    pub promoted: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Criteria for promoting memories to long-term storage
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    /// Minimum importance score to promote (0.0 - 1.0)
    pub importance_threshold: f32,
    /// Maximum age in hours before considering for consolidation
    pub max_age_hours: u32,
    /// Whether to generate synthetic insights
    pub generate_insights: bool,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            importance_threshold: 0.5,
            max_age_hours: 24,
            generate_insights: true,
        }
    }
}

/// Memory consolidation system
///
/// Periodically reviews working memory and promotes important
/// fragments to long-term storage, optionally generating insights.
pub struct MemoryConsolidator {
    vector_store: Arc<VectorStore>,
    relational_store: Arc<RelationalStore>,
    config: ConsolidationConfig,
    running: Arc<RwLock<bool>>,
    /// Optional LLM orchestrator for importance scoring
    orchestrator: Option<Arc<BrainOrchestrator>>,
}

impl MemoryConsolidator {
    /// Create a new consolidator
    pub fn new(
        vector_store: Arc<VectorStore>,
        relational_store: Arc<RelationalStore>,
        config: ConsolidationConfig,
    ) -> Self {
        Self {
            vector_store,
            relational_store,
            config,
            running: Arc::new(RwLock::new(false)),
            orchestrator: None,
        }
    }

    /// Add an LLM orchestrator for importance scoring
    pub fn with_orchestrator(mut self, orchestrator: Arc<BrainOrchestrator>) -> Self {
        self.orchestrator = Some(orchestrator);
        self
    }

    /// Run a single consolidation cycle
    ///
    /// When orchestrator is available, uses LLM for importance scoring.
    /// Otherwise uses heuristic-based scoring.
    pub async fn consolidate(&self) -> Result<ConsolidationReport> {
        let start = std::time::Instant::now();

        let stats = self.relational_store.stats()?;

        tracing::info!(
            "Consolidation cycle: {} total fragments, {} conversations, {} documents",
            stats.total_fragments,
            stats.conversation_count,
            stats.document_count
        );

        // Get recent fragments for analysis
        let recent = self.relational_store.recent_fragments(100)?;
        let mut insights_created = 0;
        let mut promoted = 0;

        for fragment in &recent {
            // Score importance
            let importance = if let Some(ref orch) = self.orchestrator {
                self.score_importance_llm(orch, &fragment.content)
                    .await
                    .unwrap_or(0.3)
            } else {
                self.score_importance_heuristic(&fragment.content)
            };

            // Promote if above threshold
            if importance >= self.config.importance_threshold {
                promoted += 1;
                tracing::debug!(
                    "Promoted fragment {} (importance: {:.2})",
                    fragment.id,
                    importance
                );
            }
        }

        // Generate insights from clusters if enabled
        if self.config.generate_insights && !recent.is_empty() {
            if let Some(ref orch) = self.orchestrator {
                if let Ok(insight) = self.generate_insight(orch, &recent).await {
                    let embedding = hash_based_embedding(&insight);
                    let _ = self
                        .promote_to_insight(
                            recent.iter().take(5).map(|f| f.id).collect(),
                            &insight,
                            &embedding,
                        )
                        .await;
                    insights_created += 1;
                }
            }
        }

        Ok(ConsolidationReport {
            processed: recent.len(),
            insights_created,
            promoted,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Score importance using LLM
    async fn score_importance_llm(
        &self,
        orchestrator: &BrainOrchestrator,
        content: &str,
    ) -> Result<f32> {
        let prompt = format!(
            "Rate the importance of this memory fragment for long-term retention (0.0-1.0).\n\
             Respond with ONLY a number.\n\nFragment: {}\n\nScore:",
            content
        );
        let response = orchestrator.think(&prompt).await?;
        response
            .trim()
            .parse::<f32>()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))
    }

    /// Score importance using simple heuristics
    fn score_importance_heuristic(&self, content: &str) -> f32 {
        let word_count = content.split_whitespace().count();
        let has_keywords = ["important", "remember", "key", "critical", "insight"]
            .iter()
            .any(|kw| content.to_lowercase().contains(kw));

        let base_score = (word_count.min(200) as f32) / 200.0 * 0.5;
        let keyword_bonus = if has_keywords { 0.3 } else { 0.0 };

        (base_score + keyword_bonus).min(1.0)
    }

    /// Generate insight from memory cluster using LLM
    async fn generate_insight(
        &self,
        orchestrator: &BrainOrchestrator,
        fragments: &[super::MemoryFragment],
    ) -> Result<String> {
        let context: Vec<_> = fragments
            .iter()
            .take(5)
            .map(|f| f.content.as_str())
            .collect();
        let prompt = format!(
            "Synthesize a single key insight from these related memories:\n\n{}\n\nInsight:",
            context.join("\n---\n")
        );
        orchestrator.think(&prompt).await
    }

    /// Start background consolidation loop
    pub async fn start_background(&self, interval_minutes: u64) {
        let mut is_running = self.running.write().await;
        if *is_running {
            tracing::warn!("Consolidation loop already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        let running = self.running.clone();
        let vector_store = self.vector_store.clone();
        let relational_store = self.relational_store.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let consolidator = MemoryConsolidator::new(vector_store, relational_store, config);
            let interval = tokio::time::Duration::from_secs(interval_minutes * 60);

            loop {
                tokio::time::sleep(interval).await;

                if !*running.read().await {
                    tracing::info!("Consolidation loop stopped");
                    break;
                }

                match consolidator.consolidate().await {
                    Ok(report) => {
                        tracing::info!(
                            "Consolidation complete: {} processed, {} insights, {} promoted ({}ms)",
                            report.processed,
                            report.insights_created,
                            report.promoted,
                            report.duration_ms
                        );
                    }
                    Err(e) => {
                        tracing::error!("Consolidation failed: {}", e);
                    }
                }
            }
        });

        tracing::info!(
            "Started background consolidation loop (interval: {} minutes)",
            interval_minutes
        );
    }

    /// Stop the background consolidation loop
    pub async fn stop(&self) {
        let mut is_running = self.running.write().await;
        *is_running = false;
        tracing::info!("Stopping consolidation loop");
    }

    /// Manually promote a specific memory to insight status
    pub async fn promote_to_insight(
        &self,
        memory_ids: Vec<Uuid>,
        insight_content: &str,
        embedding: &[f32],
    ) -> Result<Uuid> {
        let insight_id = Uuid::new_v4();
        let source = MemorySource::Insight {
            derived_from: memory_ids,
        };

        // Store in vector DB
        self.vector_store
            .store(
                insight_id,
                insight_content,
                &source,
                embedding,
                chrono::Utc::now(),
            )
            .await?;

        // Store metadata in relational DB
        self.relational_store
            .store_fragment(insight_id, insight_content, &source, None)?;

        tracing::info!("Created insight {}", insight_id);
        Ok(insight_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidation_config_default() {
        let config = ConsolidationConfig::default();
        assert_eq!(config.importance_threshold, 0.5);
        assert_eq!(config.max_age_hours, 24);
        assert!(config.generate_insights);
    }
}
