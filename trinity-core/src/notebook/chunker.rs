//! Document Chunking for TrinityNotebook
//!
//! Splits documents into overlapping chunks for embedding.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Source document ID
    pub source_id: Uuid,
    /// Index of this chunk in the document
    pub index: usize,
    /// The text content
    pub content: String,
    /// Character offset in original document
    pub start_offset: usize,
    /// Character end offset
    pub end_offset: usize,
}

/// Strategy for chunking documents
#[derive(Debug, Clone)]
pub struct ChunkingStrategy {
    /// Target size of each chunk in characters
    pub chunk_size: usize,
    /// Overlap between consecutive chunks
    pub overlap: usize,
    /// Minimum chunk size (won't create chunks smaller than this)
    pub min_chunk_size: usize,
}

impl Default for ChunkingStrategy {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            overlap: 64,
            min_chunk_size: 100,
        }
    }
}

/// Document chunker
pub struct DocumentChunker {
    strategy: ChunkingStrategy,
}

impl DocumentChunker {
    /// Create a new chunker with default strategy
    pub fn new() -> Self {
        Self {
            strategy: ChunkingStrategy::default(),
        }
    }

    /// Create a chunker with custom strategy
    pub fn with_strategy(strategy: ChunkingStrategy) -> Self {
        Self { strategy }
    }

    /// Chunk a document into overlapping pieces
    pub fn chunk(&self, source_id: Uuid, content: &str) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let total_len = chars.len();

        if total_len < self.strategy.min_chunk_size {
            // Document too small to chunk, return as single chunk
            return vec![Chunk {
                source_id,
                index: 0,
                content: content.to_string(),
                start_offset: 0,
                end_offset: content.len(),
            }];
        }

        let mut start = 0;
        let mut index = 0;

        while start < total_len {
            let end = (start + self.strategy.chunk_size).min(total_len);

            // Try to break at word boundary
            let actual_end = if end < total_len {
                self.find_break_point(&chars, start, end)
            } else {
                end
            };

            let chunk_content: String = chars[start..actual_end].iter().collect();

            // Calculate byte offsets for the original string
            let start_offset: usize = chars[..start].iter().map(|c| c.len_utf8()).sum();
            let end_offset: usize = chars[..actual_end].iter().map(|c| c.len_utf8()).sum();

            if chunk_content.len() >= self.strategy.min_chunk_size || start == 0 {
                chunks.push(Chunk {
                    source_id,
                    index,
                    content: chunk_content.trim().to_string(),
                    start_offset,
                    end_offset,
                });
                index += 1;
            }

            // Move to next chunk with overlap
            start = if actual_end >= total_len {
                total_len
            } else {
                actual_end.saturating_sub(self.strategy.overlap)
            };

            // Prevent infinite loop
            if start >= actual_end {
                break;
            }
        }

        chunks
    }

    /// Find a good break point (word boundary) near the target position
    fn find_break_point(&self, chars: &[char], start: usize, target: usize) -> usize {
        // Look back from target to find a space or punctuation
        for i in (start..target).rev() {
            if chars[i].is_whitespace() || chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
                return i + 1;
            }
        }
        // No good break point found, use target
        target
    }
}

impl Default for DocumentChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_small_doc() {
        let chunker = DocumentChunker::new();
        let content = "This is a small document.";
        let chunks = chunker.chunk(Uuid::new_v4(), content);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, content);
    }

    #[test]
    fn test_chunking_large_doc() {
        let chunker = DocumentChunker::with_strategy(ChunkingStrategy {
            chunk_size: 50,
            overlap: 10,
            min_chunk_size: 20,
        });

        let content = "This is a longer document that should be split into multiple chunks. \
                       Each chunk will have some overlap with the previous one to maintain context.";

        let chunks = chunker.chunk(Uuid::new_v4(), content);

        assert!(chunks.len() > 1);
        assert!(chunks[0].index == 0);
    }
}
