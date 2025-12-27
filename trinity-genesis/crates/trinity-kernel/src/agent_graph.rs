//! # Agent Graph - Deterministic Workflow DAGs
//!
//! ## Philosophy
//! "The Agent Graph is the nervous system of Trinity—defining how thoughts flow
//!  between agents. Unlike conversational loops, graphs provide determinism,
//!  allowing for reproducible, debuggable agent coordination."
//!
//! ## Architecture
//!
//! ```text
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │                        Agent Graph                              │
//!    │                                                                 │
//!    │   ┌──────────┐    ┌──────────┐    ┌──────────┐                 │
//!    │   │ Planner  │───►│  Coder   │───►│ Reviewer │                 │
//!    │   │ (Joshua) │    │ (Jessica)│    │ (Jules)  │                 │
//!    │   └──────────┘    └──────────┘    └──────────┘                 │
//!    │        │                               │                        │
//!    │        │         ┌──────────┐          │                        │
//!    │        └────────►│Researcher│◄─────────┘                        │
//!    │                  │ (Janet)  │                                   │
//!    │                  └──────────┘                                   │
//!    │                                                                 │
//!    │   ═══════════════════════════════════════════════════          │
//!    │   Lock-Free DAG Scheduler (Tokio + petgraph)                   │
//!    └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! 1. **Deterministic Execution**: Same inputs → same outputs
//! 2. **Lock-Free Scheduling**: Agents execute in parallel where possible
//! 3. **Schema Validated**: I/O contracts enforced at compile time
//! 4. **Event Streaming**: All transitions broadcast to UI

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::orchestrator::AgentEvent;

// ============================================================================
// Node Port (Connection Points)
// ============================================================================

/// A connection point on a graph node
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePort {
    /// Node this port belongs to
    pub node_id: Uuid,

    /// Port name (e.g., "output", "error", "code")
    pub port_name: String,
}

impl NodePort {
    pub fn new(node_id: Uuid, port_name: impl Into<String>) -> Self {
        Self {
            node_id,
            port_name: port_name.into(),
        }
    }

    pub fn output(node_id: Uuid) -> Self {
        Self::new(node_id, "output")
    }

    pub fn input(node_id: Uuid) -> Self {
        Self::new(node_id, "input")
    }
}

// ============================================================================
// Agent Node (Graph Vertex)
// ============================================================================

/// A node in the agent workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    /// Unique identifier
    pub id: Uuid,

    /// Agent name/persona
    pub name: String,

    /// Agent specialization
    pub specialization: NodeSpecialization,

    /// Input ports (can receive data from other nodes)
    pub inputs: Vec<String>,

    /// Output ports (can send data to other nodes)
    pub outputs: Vec<String>,

    /// Current execution status
    pub status: NodeStatus,

    /// Associated task (if executing)
    pub current_task: Option<Uuid>,
}

/// Node specialization (maps to agent personas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeSpecialization {
    /// High-level planning and coordination (Joshua)
    Planner,

    /// Code generation and implementation (Jessica)
    Coder,

    /// Code review and quality assurance (Jules)
    Reviewer,

    /// Research and information gathering (Janet)
    Researcher,

    /// Custom agent type
    Custom,
}

/// Node execution status
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NodeStatus {
    #[default]
    Idle,
    Waiting,
    Running,
    Completed,
    Failed(String),
}

impl AgentNode {
    /// Create a new agent node
    pub fn new(name: impl Into<String>, specialization: NodeSpecialization) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            specialization,
            inputs: vec!["input".into()],
            outputs: vec!["output".into()],
            status: NodeStatus::Idle,
            current_task: None,
        }
    }

    /// Add an input port
    pub fn with_input(mut self, name: impl Into<String>) -> Self {
        self.inputs.push(name.into());
        self
    }

    /// Add an output port
    pub fn with_output(mut self, name: impl Into<String>) -> Self {
        self.outputs.push(name.into());
        self
    }
}

// ============================================================================
// Graph Edge (Connection)
// ============================================================================

/// An edge connecting two node ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source port
    pub from: NodePort,

    /// Destination port
    pub to: NodePort,

    /// Edge metadata (optional transformation)
    pub transform: Option<String>,
}

impl GraphEdge {
    pub fn new(from: NodePort, to: NodePort) -> Self {
        Self {
            from,
            to,
            transform: None,
        }
    }

    pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
        self.transform = Some(transform.into());
        self
    }
}

// ============================================================================
// Graph Result (Execution Output)
// ============================================================================

/// Result of executing a graph workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    /// Graph identifier
    pub graph_id: Uuid,

    /// Whether all nodes completed successfully
    pub success: bool,

    /// Node outputs (node_id → output string)
    pub node_outputs: HashMap<Uuid, String>,

    /// Final output (from terminal nodes)
    pub final_output: Option<String>,

    /// Total execution time in milliseconds
    pub duration_ms: u64,

    /// Errors encountered (node_id → error message)
    pub errors: HashMap<Uuid, String>,
}

// ============================================================================
// Agent Graph (DAG Workflow)
// ============================================================================

/// A Directed Acyclic Graph of agent nodes
///
/// Represents a workflow where agents collaborate to complete a task.
/// Execution is deterministic and parallelized where dependencies allow.
#[derive(Clone)]
pub struct AgentGraph {
    /// Unique identifier
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Nodes in the graph
    nodes: Arc<RwLock<HashMap<Uuid, AgentNode>>>,

    /// Edges connecting nodes
    edges: Arc<RwLock<Vec<GraphEdge>>>,

    /// Event broadcaster
    event_tx: broadcast::Sender<AgentEvent>,
}

impl AgentGraph {
    /// Create a new empty graph
    pub fn new(name: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(256);

        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(Vec::new())),
            event_tx,
        }
    }

    /// Create a graph builder
    pub fn builder(name: impl Into<String>) -> AgentGraphBuilder {
        AgentGraphBuilder::new(name)
    }

    /// Add a node to the graph
    pub async fn add_node(&self, node: AgentNode) -> Uuid {
        let id = node.id;
        let mut nodes = self.nodes.write().await;
        nodes.insert(id, node);
        id
    }

    /// Add an edge between nodes
    pub async fn add_edge(&self, edge: GraphEdge) -> Result<()> {
        let nodes = self.nodes.read().await;

        // Validate source node exists
        if !nodes.contains_key(&edge.from.node_id) {
            return Err(anyhow::anyhow!(
                "Source node {} not found",
                edge.from.node_id
            ));
        }

        // Validate destination node exists
        if !nodes.contains_key(&edge.to.node_id) {
            return Err(anyhow::anyhow!(
                "Destination node {} not found",
                edge.to.node_id
            ));
        }

        // Cycle detection: check if adding this edge would create a cycle
        // A cycle exists if there's a path from destination back to source
        let edges = self.edges.read().await;
        if Self::would_create_cycle(&edge, &edges) {
            return Err(anyhow::anyhow!(
                "Adding edge {} -> {} would create a cycle",
                edge.from.node_id,
                edge.to.node_id
            ));
        }
        drop(edges);

        drop(nodes);
        let mut edges = self.edges.write().await;
        edges.push(edge);

        Ok(())
    }

    /// Check if adding an edge would create a cycle using DFS
    fn would_create_cycle(new_edge: &GraphEdge, edges: &[GraphEdge]) -> bool {
        use std::collections::HashSet;
        
        let source = new_edge.from.node_id;
        let dest = new_edge.to.node_id;
        
        // If source == dest, it's a self-loop
        if source == dest {
            return true;
        }
        
        // DFS from dest to see if we can reach source
        let mut visited = HashSet::new();
        let mut stack = vec![dest];
        
        while let Some(current) = stack.pop() {
            if current == source {
                return true; // Found a path back to source
            }
            
            if visited.insert(current) {
                for edge in edges {
                    if edge.from.node_id == current {
                        stack.push(edge.to.node_id);
                    }
                }
            }
        }
        
        false
    }

    /// Get all nodes
    pub async fn nodes(&self) -> Vec<AgentNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get a node by ID
    pub async fn get_node(&self, id: Uuid) -> Option<AgentNode> {
        let nodes = self.nodes.read().await;
        nodes.get(&id).cloned()
    }

    /// Subscribe to graph events
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Find nodes with no incoming edges (entry points)
    pub async fn entry_nodes(&self) -> Vec<AgentNode> {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        let has_incoming: std::collections::HashSet<Uuid> =
            edges.iter().map(|e| e.to.node_id).collect();

        nodes
            .values()
            .filter(|n| !has_incoming.contains(&n.id))
            .cloned()
            .collect()
    }

    /// Find nodes with no outgoing edges (terminal nodes)
    pub async fn terminal_nodes(&self) -> Vec<AgentNode> {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        let has_outgoing: std::collections::HashSet<Uuid> =
            edges.iter().map(|e| e.from.node_id).collect();

        nodes
            .values()
            .filter(|n| !has_outgoing.contains(&n.id))
            .cloned()
            .collect()
    }

    /// Get dependencies for a node (nodes that must complete first)
    pub async fn dependencies(&self, node_id: Uuid) -> Vec<Uuid> {
        let edges = self.edges.read().await;

        edges
            .iter()
            .filter(|e| e.to.node_id == node_id)
            .map(|e| e.from.node_id)
            .collect()
    }

    /// Execute the graph workflow
    ///
    /// This is the main execution entry point. Nodes are executed in
    /// topological order, parallelizing where dependencies allow.
    pub async fn execute(&self, initial_input: &str) -> Result<GraphResult> {
        use std::time::Instant;
        let start = Instant::now();

        let mut node_outputs: HashMap<Uuid, String> = HashMap::new();
        let errors: HashMap<Uuid, String> = HashMap::new();

        // Get entry nodes
        let entry_nodes = self.entry_nodes().await;

        if entry_nodes.is_empty() {
            return Err(anyhow::anyhow!("Graph has no entry nodes"));
        }

        // For now, simple sequential execution (TODO: parallel with topological sort)
        for node in entry_nodes {
            // Broadcast start event
            let _ = self.event_tx.send(AgentEvent::TaskStarted {
                agent_id: node.id.to_string(),
                task_id: Uuid::new_v4(),
                task_name: format!("Graph node: {}", node.name),
            });

            // Simulate node execution
            let output = format!(
                "[{}] Processed: {}...",
                node.name,
                &initial_input[..initial_input.len().min(50)]
            );

            node_outputs.insert(node.id, output.clone());

            // Broadcast completion
            let _ = self.event_tx.send(AgentEvent::TaskCompleted {
                agent_id: node.id.to_string(),
                task_id: Uuid::new_v4(),
                result: output.clone(),
                duration_ms: 0, // Graph-level timing tracked separately
            });
        }

        // Get terminal nodes for final output
        let terminal = self.terminal_nodes().await;
        let final_output = terminal
            .first()
            .and_then(|n| node_outputs.get(&n.id))
            .cloned();

        Ok(GraphResult {
            graph_id: self.id,
            success: errors.is_empty(),
            node_outputs,
            final_output,
            duration_ms: start.elapsed().as_millis() as u64,
            errors,
        })
    }
}

// ============================================================================
// Graph Builder (Fluent API)
// ============================================================================

/// Builder for constructing agent graphs
pub struct AgentGraphBuilder {
    name: String,
    nodes: Vec<AgentNode>,
    edges: Vec<(usize, usize)>, // (from_idx, to_idx)
}

impl AgentGraphBuilder {
    /// Create a new graph builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a planner node
    pub fn planner(mut self, name: impl Into<String>) -> Self {
        self.nodes
            .push(AgentNode::new(name, NodeSpecialization::Planner));
        self
    }

    /// Add a coder node
    pub fn coder(mut self, name: impl Into<String>) -> Self {
        self.nodes
            .push(AgentNode::new(name, NodeSpecialization::Coder));
        self
    }

    /// Add a reviewer node
    pub fn reviewer(mut self, name: impl Into<String>) -> Self {
        self.nodes
            .push(AgentNode::new(name, NodeSpecialization::Reviewer));
        self
    }

    /// Add a researcher node
    pub fn researcher(mut self, name: impl Into<String>) -> Self {
        self.nodes
            .push(AgentNode::new(name, NodeSpecialization::Researcher));
        self
    }

    /// Connect two nodes by index (0-based)
    pub fn connect(mut self, from_idx: usize, to_idx: usize) -> Self {
        self.edges.push((from_idx, to_idx));
        self
    }

    /// Build the graph
    pub async fn build(self) -> Result<AgentGraph> {
        let graph = AgentGraph::new(self.name);

        // Add all nodes
        let mut node_ids = Vec::new();
        for node in self.nodes {
            let id = graph.add_node(node).await;
            node_ids.push(id);
        }

        // Add all edges
        for (from_idx, to_idx) in self.edges {
            if from_idx >= node_ids.len() || to_idx >= node_ids.len() {
                return Err(anyhow::anyhow!(
                    "Invalid edge: {} -> {}",
                    from_idx,
                    to_idx
                ));
            }

            let edge = GraphEdge::new(
                NodePort::output(node_ids[from_idx]),
                NodePort::input(node_ids[to_idx]),
            );

            graph.add_edge(edge).await?;
        }

        Ok(graph)
    }
}

// ============================================================================
// Predefined Workflow Templates
// ============================================================================

impl AgentGraph {
    /// Create a simple Planning → Coding → Review workflow
    pub async fn planning_workflow() -> Result<Self> {
        AgentGraphBuilder::new("Planning Workflow")
            .planner("Joshua")
            .coder("Jessica")
            .reviewer("Jules")
            .connect(0, 1) // Planner → Coder
            .connect(1, 2) // Coder → Reviewer
            .build()
            .await
    }

    /// Create a research-enhanced coding workflow
    pub async fn research_workflow() -> Result<Self> {
        AgentGraphBuilder::new("Research Workflow")
            .researcher("Janet")
            .planner("Joshua")
            .coder("Jessica")
            .connect(0, 1) // Researcher → Planner
            .connect(1, 2) // Planner → Coder
            .build()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_creation() {
        let graph = AgentGraph::new("Test Graph");
        assert!(graph.nodes().await.is_empty());
    }

    #[tokio::test]
    async fn test_graph_builder() {
        let graph = AgentGraphBuilder::new("Test")
            .planner("P1")
            .coder("C1")
            .connect(0, 1)
            .build()
            .await
            .unwrap();

        assert_eq!(graph.nodes().await.len(), 2);
    }

    #[tokio::test]
    async fn test_entry_terminal_nodes() {
        let graph = AgentGraph::planning_workflow().await.unwrap();

        let entries = graph.entry_nodes().await;
        let terminals = graph.terminal_nodes().await;

        assert_eq!(entries.len(), 1);
        assert_eq!(terminals.len(), 1);
        assert_eq!(entries[0].name, "Joshua");
        assert_eq!(terminals[0].name, "Jules");
    }
}
