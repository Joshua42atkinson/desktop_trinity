//! Agent Module - ECS-based AI Agent Runtime
//!
//! Provides the swarm agent architecture using Bevy ECS.

mod components;
mod executor;
mod router;

pub mod output_parser;
pub mod prompts;
pub mod specialized;
pub mod symbol_registry;
pub mod tools;

pub use components::*;
pub use executor::*;
pub use output_parser::{parse_output, OutputParser, ParsedOutput, ParsedToolCall};
pub use router::*;
pub use symbol_registry::{SymbolInfo, SymbolKind, SymbolMatch, SymbolRegistry};
