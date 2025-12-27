use crate::types::MemoryFact;

#[tarpc::service]
pub trait MemoryService {
    /// Store a new fact in long-term memory
    async fn remember(content: String) -> String; // returns ID

    /// Recall relevant facts based on a query
    async fn recall(query: String) -> Vec<MemoryFact>;
}
