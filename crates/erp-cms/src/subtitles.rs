use serde::{Deserialize, Serialize};

/// Timestamped subtitle phrase segment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleSegment {
    pub start_mark: f64,
    pub end_mark: f64,
    pub phrase: String,
    pub embedding: Vec<f32>,
}

/// SurrealDB v3 Vector Index DDL and Semantic Subtitle Search (Milestone 5.3).
pub struct SubtitleSearchEngine;

impl SubtitleSearchEngine {
    /// Emits SurrealDB v3 HNSW Vector Index DDL statements.
    #[must_use]
    pub fn compile_vector_index_ddl() -> Vec<String> {
        vec![
            "DEFINE TABLE media_transcripts SCHEMAFULL;".to_string(),
            "DEFINE FIELD video_id ON TABLE media_transcripts TYPE record<media>;".to_string(),
            "DEFINE FIELD start_mark ON TABLE media_transcripts TYPE number;".to_string(),
            "DEFINE FIELD end_mark ON TABLE media_transcripts TYPE number;".to_string(),
            "DEFINE FIELD phrase ON TABLE media_transcripts TYPE string;".to_string(),
            "DEFINE FIELD embedding ON TABLE media_transcripts TYPE array<number, 1536>;".to_string(),
            "DEFINE INDEX idx_transcripts_vector ON TABLE media_transcripts FIELDS embedding TYPE HNSW DISTANCE COSINE DIMENSION 1536;".to_string(),
        ]
    }

    /// Computes cosine similarity between two vector embeddings: $\frac{A \cdot B}{\|A\| \|B\|}$.
    #[must_use]
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (x, y) in a.iter().zip(b.iter()) {
            dot_product += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a.sqrt() * norm_b.sqrt())
        }
    }
}
