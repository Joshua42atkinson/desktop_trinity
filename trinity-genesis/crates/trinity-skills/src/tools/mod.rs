// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # Tools Module (Self-Coding Capabilities)
//!
//! Provides file and shell operation tools for Trinity's autonomous work.
//!
//! ## Components
//! - `executor`: File read/write, shell commands

pub mod executor;

pub use executor::{ExecutionLogEntry, ExecutorConfig, ToolExecutor};
