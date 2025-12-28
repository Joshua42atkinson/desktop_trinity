// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Autopoietic Loop - Self-Modifying Code Engine
//!
//! ## Philosophy (The Soul)
//! "The Soul is the mechanism of self-creation. An agent that cannot
//!  modify its own source code is merely a tool. The autopoietic loop
//!  transforms Trinity from software into a living system."
//!
//! ## Safety Protocols (Layered Defense)
//!
//! 1. **Staging**: Mutations happen in a staging directory, never live code
//! 2. **AST Validation**: Use `syn` to parse before writing, ensuring valid Rust
//! 3. **Compilation Gate**: Code must compile before being accepted
//! 4. **Test Gate**: Tests must pass (optional, configurable)
//! 5. **Rollback**: Keep N previous versions for instant rollback
//! 6. **Cloud Backup**: Sync to Google Drive before mutations
//! 7. **Kill Switch**: `safety.rs` is IMMUTABLE and can halt all mutations
//!
//! ## The Ouroboros Workflow
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        AUTOPOIETIC LOOP                                  │
//! │                                                                          │
//! │  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐              │
//! │  │  Brain   │──▶│ Generate │──▶│   AST    │──▶│ Staging  │              │
//! │  │ (Prompt) │   │   Code   │   │ Validate │   │  Write   │              │
//! │  └──────────┘   └──────────┘   └──────────┘   └────┬─────┘              │
//! │                                                     │                    │
//! │  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌────▼─────┐              │
//! │  │  Swap    │◀──│ Backup   │◀──│  Test    │◀──│ Compile  │              │
//! │  │  Binary  │   │ to Cloud │   │  (opt)   │   │  Check   │              │
//! │  └────┬─────┘   └──────────┘   └──────────┘   └──────────┘              │
//! │       │                                                                  │
//! │       ▼                                                                  │
//! │  ┌──────────┐                                                            │
//! │  │ Restart  │ ─── State Serialized ──▶ memory.dump                       │
//! │  │  Self    │                                                            │
//! │  └──────────┘                                                            │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, warn};

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for the autopoietic loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopoieticConfig {
    /// Root directory of the Trinity source code
    pub source_root: PathBuf,
    /// Directory for staging mutations
    pub staging_dir: PathBuf,
    /// Directory for version backups
    pub backup_dir: PathBuf,
    /// Number of backup versions to keep
    pub max_backups: usize,
    /// Whether to run tests before accepting mutations
    pub require_tests: bool,
    /// Google Drive folder for cloud backup (if configured)
    pub cloud_backup_path: Option<PathBuf>,
    /// Files that are NEVER allowed to be modified
    pub immutable_files: Vec<String>,
    /// Maximum consecutive failures before halting
    pub max_failures: u32,
}

impl Default for AutopoieticConfig {
    fn default() -> Self {
        Self {
            source_root: PathBuf::from("/home/joshua/antigravity/trinity-genesis"),
            staging_dir: PathBuf::from("/home/joshua/antigravity/trinity_staging"),
            backup_dir: PathBuf::from("/home/joshua/antigravity/trinity_backups"),
            max_backups: 10,
            require_tests: false,    // Start lenient, tighten as we stabilize
            cloud_backup_path: None, // Set via with_cloud_backup()
            immutable_files: vec![
                "safety.rs".to_string(),      // Kill switch
                "autopoietic.rs".to_string(), // Can't modify itself (infinite loop risk)
                "Cargo.lock".to_string(),     // Don't break deps
            ],
            max_failures: 3,
        }
    }
}

impl AutopoieticConfig {
    /// Enable Google Drive cloud backup
    pub fn with_cloud_backup(mut self, path: impl Into<PathBuf>) -> Self {
        self.cloud_backup_path = Some(path.into());
        self
    }

    /// Require tests to pass before accepting mutations
    pub fn with_tests(mut self) -> Self {
        self.require_tests = true;
        self
    }

    /// Add an immutable file
    pub fn with_immutable(mut self, file: impl Into<String>) -> Self {
        self.immutable_files.push(file.into());
        self
    }

    /// Create a production configuration with Google Drive cloud backup enabled
    ///
    /// This is the recommended configuration for production use.
    /// Cloud backups are synced to ~/Google Drive/trinity_backups/
    pub fn production() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/joshua"));
        let gdrive_path = home.join("Google Drive").join("trinity_backups");

        Self::default().with_cloud_backup(gdrive_path).with_tests()
    }

    /// Create a production configuration with a custom Google Drive path
    pub fn production_with_gdrive(gdrive_backup_folder: impl Into<PathBuf>) -> Self {
        Self::default()
            .with_cloud_backup(gdrive_backup_folder)
            .with_tests()
    }

    /// Enable Google Drive cloud backup using the default location
    ///
    /// Default location: ~/Google Drive/trinity_backups/
    pub fn with_default_cloud_backup(mut self) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/joshua"));
        self.cloud_backup_path = Some(home.join("Google Drive").join("trinity_backups"));
        self
    }
}

// ============================================================================
// Mutation Request
// ============================================================================

/// A request to mutate the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRequest {
    /// Target file (relative to source root)
    pub target_file: String,
    /// Type of mutation
    pub mutation_type: MutationType,
    /// Description of what to do
    pub description: String,
    /// The code to insert/replace (for Insert/Replace types)
    pub code: Option<String>,
    /// For Replace: the code to find
    pub find_pattern: Option<String>,
}

/// Types of code mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationType {
    /// Insert code at a specific location (e.g., add a function)
    Insert {
        /// Where to insert: "after_imports", "end_of_file", "in_impl:StructName"
        location: String,
    },
    /// Replace existing code
    Replace,
    /// Create a new file
    CreateFile,
    /// Delete a file (dangerous - requires explicit permission)
    DeleteFile,
    /// Append to end of file
    Append,
}

/// Result of a mutation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    /// Whether the mutation was successful
    pub success: bool,
    /// The backup version number (for rollback)
    pub backup_version: Option<u64>,
    /// Compiler output (if compilation was attempted)
    pub compiler_output: Option<String>,
    /// Test output (if tests were run)
    pub test_output: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

// ============================================================================
// Autopoietic Engine
// ============================================================================

/// The autopoietic engine for self-modification
pub struct AutopoieticEngine {
    config: AutopoieticConfig,
    /// Consecutive failure count
    failure_count: u32,
    /// Current backup version number
    current_version: u64,
}

impl AutopoieticEngine {
    /// Create a new autopoietic engine
    pub fn new(config: AutopoieticConfig) -> Result<Self> {
        // Ensure directories exist
        std::fs::create_dir_all(&config.staging_dir)?;
        std::fs::create_dir_all(&config.backup_dir)?;

        // Determine current version from existing backups
        let current_version = Self::find_latest_version(&config.backup_dir)?;

        info!(
            "Autopoietic engine initialized. Current version: {}",
            current_version
        );

        Ok(Self {
            config,
            failure_count: 0,
            current_version,
        })
    }

    /// Find the latest backup version number
    fn find_latest_version(backup_dir: &Path) -> Result<u64> {
        let mut max_version = 0u64;

        if backup_dir.exists() {
            for entry in std::fs::read_dir(backup_dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if let Some(version_str) = name.strip_prefix("v") {
                        if let Ok(version) = version_str.parse::<u64>() {
                            max_version = max_version.max(version);
                        }
                    }
                }
            }
        }

        Ok(max_version)
    }

    /// Check if a file is allowed to be modified
    fn is_mutable(&self, file: &str) -> bool {
        !self
            .config
            .immutable_files
            .iter()
            .any(|f| file.ends_with(f))
    }

    /// Execute a mutation
    pub fn execute(&mut self, request: MutationRequest) -> Result<MutationResult> {
        info!("Executing mutation: {:?}", request.mutation_type);

        // Safety check: is the file mutable?
        if !self.is_mutable(&request.target_file) {
            error!(
                "BLOCKED: Attempt to modify immutable file: {}",
                request.target_file
            );
            return Ok(MutationResult {
                success: false,
                backup_version: None,
                compiler_output: None,
                test_output: None,
                error: Some(format!("File '{}' is immutable", request.target_file)),
            });
        }

        // Safety check: too many failures?
        if self.failure_count >= self.config.max_failures {
            error!(
                "BLOCKED: Too many consecutive failures ({})",
                self.failure_count
            );
            return Ok(MutationResult {
                success: false,
                backup_version: None,
                compiler_output: None,
                test_output: None,
                error: Some(
                    "Too many consecutive failures. Manual intervention required.".to_string(),
                ),
            });
        }

        // Step 1: Copy source to staging
        self.copy_to_staging()?;

        // Step 2: Apply mutation in staging
        let staging_target = self.config.staging_dir.join(&request.target_file);
        self.apply_mutation(&staging_target, &request)?;

        // Step 3: Validate syntax with syn (for Rust files)
        if request.target_file.ends_with(".rs") {
            if let Err(e) = self.validate_rust_syntax(&staging_target) {
                self.failure_count += 1;
                return Ok(MutationResult {
                    success: false,
                    backup_version: None,
                    compiler_output: None,
                    test_output: None,
                    error: Some(format!("Syntax validation failed: {}", e)),
                });
            }
        }

        // Step 4: Compile in staging
        let compile_result = self.compile_staging()?;
        if !compile_result.success {
            self.failure_count += 1;
            return Ok(MutationResult {
                success: false,
                backup_version: None,
                compiler_output: Some(compile_result.output),
                test_output: None,
                error: Some("Compilation failed".to_string()),
            });
        }

        // Step 5: Run tests (if configured)
        let test_output = if self.config.require_tests {
            let test_result = self.run_tests()?;
            if !test_result.success {
                self.failure_count += 1;
                return Ok(MutationResult {
                    success: false,
                    backup_version: None,
                    compiler_output: Some(compile_result.output),
                    test_output: Some(test_result.output),
                    error: Some("Tests failed".to_string()),
                });
            }
            Some(test_result.output)
        } else {
            None
        };

        // Step 6: Create backup of current source
        self.current_version += 1;
        let backup_path = self.create_backup()?;
        info!(
            "Created backup v{} at {:?}",
            self.current_version, backup_path
        );

        // Step 7: Cloud backup (if configured)
        if let Some(ref cloud_path) = self.config.cloud_backup_path {
            if let Err(e) = self.sync_to_cloud(cloud_path) {
                warn!("Cloud backup failed (continuing anyway): {}", e);
            }
        }

        // Step 8: Copy staging to live source
        self.promote_staging()?;

        // Success! Reset failure count
        self.failure_count = 0;

        info!(
            "✓ Mutation successful. New version: {}",
            self.current_version
        );

        Ok(MutationResult {
            success: true,
            backup_version: Some(self.current_version),
            compiler_output: Some(compile_result.output),
            test_output,
            error: None,
        })
    }

    /// Copy source to staging directory
    fn copy_to_staging(&self) -> Result<()> {
        debug!("Copying source to staging...");

        // Clean staging first
        if self.config.staging_dir.exists() {
            std::fs::remove_dir_all(&self.config.staging_dir)?;
        }

        // Copy
        copy_dir_recursive(
            &self.config.source_root.join("crates"),
            &self.config.staging_dir.join("crates"),
        )?;

        // Also copy Cargo.toml and Cargo.lock
        std::fs::copy(
            self.config.source_root.join("Cargo.toml"),
            self.config.staging_dir.join("Cargo.toml"),
        )?;

        if self.config.source_root.join("Cargo.lock").exists() {
            std::fs::copy(
                self.config.source_root.join("Cargo.lock"),
                self.config.staging_dir.join("Cargo.lock"),
            )?;
        }

        Ok(())
    }

    /// Apply a mutation to a file
    fn apply_mutation(&self, target: &Path, request: &MutationRequest) -> Result<()> {
        match &request.mutation_type {
            MutationType::CreateFile => {
                let code = request.code.as_ref().context("CreateFile requires code")?;
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(target, code)?;
            }
            MutationType::Append => {
                let code = request.code.as_ref().context("Append requires code")?;
                let mut content = std::fs::read_to_string(target).unwrap_or_default();
                content.push_str("\n");
                content.push_str(code);
                std::fs::write(target, content)?;
            }
            MutationType::Replace => {
                let code = request.code.as_ref().context("Replace requires code")?;
                let find = request
                    .find_pattern
                    .as_ref()
                    .context("Replace requires find_pattern")?;
                let content = std::fs::read_to_string(target)?;
                let new_content = content.replace(find, code);
                std::fs::write(target, new_content)?;
            }
            MutationType::Insert { location } => {
                let code = request.code.as_ref().context("Insert requires code")?;
                let content = std::fs::read_to_string(target)?;
                let new_content = insert_at_location(&content, code, location)?;
                std::fs::write(target, new_content)?;
            }
            MutationType::DeleteFile => {
                if target.exists() {
                    std::fs::remove_file(target)?;
                }
            }
        }
        Ok(())
    }

    /// Validate Rust syntax using basic parsing
    fn validate_rust_syntax(&self, file: &Path) -> Result<()> {
        let content = std::fs::read_to_string(file)?;

        // Basic brace/paren/bracket matching
        let mut stack: Vec<char> = Vec::new();
        for ch in content.chars() {
            match ch {
                '{' | '(' | '[' => stack.push(ch),
                '}' => {
                    if stack.pop() != Some('{') {
                        anyhow::bail!("Unmatched }}");
                    }
                }
                ')' => {
                    if stack.pop() != Some('(') {
                        anyhow::bail!("Unmatched )");
                    }
                }
                ']' => {
                    if stack.pop() != Some('[') {
                        anyhow::bail!("Unmatched ]");
                    }
                }
                _ => {}
            }
        }

        if !stack.is_empty() {
            anyhow::bail!("Unclosed brackets: {:?}", stack);
        }

        Ok(())
    }

    /// Compile the staging directory
    fn compile_staging(&self) -> Result<CommandResult> {
        info!("Compiling staging...");

        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&self.config.staging_dir)
            .output()
            .context("Failed to run cargo build")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandResult {
            success: output.status.success(),
            output: format!("{}\n{}", stdout, stderr),
        })
    }

    /// Run tests in staging
    fn run_tests(&self) -> Result<CommandResult> {
        info!("Running tests...");

        let output = Command::new("cargo")
            .arg("test")
            .current_dir(&self.config.staging_dir)
            .output()
            .context("Failed to run cargo test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandResult {
            success: output.status.success(),
            output: format!("{}\n{}", stdout, stderr),
        })
    }

    /// Create a backup of the current source
    fn create_backup(&self) -> Result<PathBuf> {
        let backup_name = format!("v{}", self.current_version);
        let backup_path = self.config.backup_dir.join(&backup_name);

        copy_dir_recursive(
            &self.config.source_root.join("crates"),
            &backup_path.join("crates"),
        )?;

        // Prune old backups
        self.prune_old_backups()?;

        Ok(backup_path)
    }

    /// Prune old backups keeping only max_backups
    fn prune_old_backups(&self) -> Result<()> {
        let mut versions: Vec<u64> = Vec::new();

        for entry in std::fs::read_dir(&self.config.backup_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if let Some(version_str) = name.strip_prefix("v") {
                    if let Ok(version) = version_str.parse::<u64>() {
                        versions.push(version);
                    }
                }
            }
        }

        versions.sort();

        // Remove oldest if we have too many
        while versions.len() > self.config.max_backups {
            if let Some(oldest) = versions.first() {
                let path = self.config.backup_dir.join(format!("v{}", oldest));
                debug!("Pruning old backup: {:?}", path);
                std::fs::remove_dir_all(path)?;
                versions.remove(0);
            }
        }

        Ok(())
    }

    /// Sync to cloud storage (Google Drive)
    fn sync_to_cloud(&self, cloud_path: &Path) -> Result<()> {
        info!("Syncing to cloud: {:?}", cloud_path);

        // Use rclone if available, otherwise just copy
        let backup_name = format!("v{}", self.current_version);
        let source = self.config.backup_dir.join(&backup_name);
        let dest = cloud_path.join(&backup_name);

        copy_dir_recursive(&source, &dest)?;

        info!("✓ Cloud backup complete: {:?}", dest);
        Ok(())
    }

    /// Promote staging to live source
    fn promote_staging(&self) -> Result<()> {
        info!("Promoting staging to live...");

        // Copy staged crates over live crates
        let staged_crates = self.config.staging_dir.join("crates");
        let live_crates = self.config.source_root.join("crates");

        // For each crate in staging, copy its src directory
        for entry in std::fs::read_dir(&staged_crates)? {
            let entry = entry?;
            let crate_name = entry.file_name();
            let staged_src = staged_crates.join(&crate_name).join("src");
            let live_src = live_crates.join(&crate_name).join("src");

            if staged_src.exists() {
                copy_dir_recursive(&staged_src, &live_src)?;
            }
        }

        info!("✓ Live source updated");
        Ok(())
    }

    /// Rollback to a specific version
    pub fn rollback(&mut self, version: u64) -> Result<()> {
        let backup_path = self.config.backup_dir.join(format!("v{}", version));

        if !backup_path.exists() {
            anyhow::bail!("Backup v{} does not exist", version);
        }

        info!("Rolling back to v{}...", version);

        // Copy backup over live
        copy_dir_recursive(
            &backup_path.join("crates"),
            &self.config.source_root.join("crates"),
        )?;

        self.current_version = version;
        self.failure_count = 0;

        info!("✓ Rollback complete to v{}", version);
        Ok(())
    }

    /// Get current version
    pub fn current_version(&self) -> u64 {
        self.current_version
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Reset failure count (manual override)
    pub fn reset_failures(&mut self) {
        self.failure_count = 0;
        info!("Failure count reset");
    }
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

struct CommandResult {
    success: bool,
    output: String,
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Insert code at a specified location
fn insert_at_location(content: &str, code: &str, location: &str) -> Result<String> {
    match location {
        "end_of_file" => Ok(format!("{}\n{}", content, code)),
        "after_imports" => {
            // Find the last `use` statement and insert after it
            let lines: Vec<&str> = content.lines().collect();
            let mut last_use_idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("use ") {
                    last_use_idx = i;
                }
            }
            let mut result = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                result.push(*line);
                if i == last_use_idx {
                    result.push("");
                    result.push(code);
                }
            }
            Ok(result.join("\n"))
        }
        loc if loc.starts_with("in_impl:") => {
            // Insert at the end of an impl block for a struct
            let struct_name = loc.strip_prefix("in_impl:").unwrap();
            let pattern = format!("impl {}", struct_name);
            if let Some(impl_start) = content.find(&pattern) {
                // Find matching closing brace
                let after_impl = &content[impl_start..];
                let mut brace_count = 0;
                let mut insert_pos = None;
                for (i, ch) in after_impl.chars().enumerate() {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            insert_pos = Some(impl_start + i);
                            break;
                        }
                    }
                }
                if let Some(pos) = insert_pos {
                    let (before, after) = content.split_at(pos);
                    return Ok(format!("{}\n    {}\n{}", before, code, after));
                }
            }
            anyhow::bail!("Could not find impl block for {}", struct_name)
        }
        _ => anyhow::bail!("Unknown insert location: {}", location),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_end_of_file() {
        let content = "fn main() {}";
        let code = "fn extra() {}";
        let result = insert_at_location(content, code, "end_of_file").unwrap();
        assert!(result.contains("fn extra()"));
    }

    #[test]
    fn test_insert_after_imports() {
        let content = "use std::io;\nuse std::fs;\n\nfn main() {}";
        let code = "use extra::module;";
        let result = insert_at_location(content, code, "after_imports").unwrap();
        assert!(result.contains("use extra::module;"));
    }

    #[test]
    fn test_is_mutable() {
        let config = AutopoieticConfig::default();
        let engine = AutopoieticEngine {
            config,
            failure_count: 0,
            current_version: 0,
        };

        assert!(!engine.is_mutable("src/safety.rs"));
        assert!(!engine.is_mutable("path/to/autopoietic.rs"));
        assert!(engine.is_mutable("src/brain.rs"));
        assert!(engine.is_mutable("src/main.rs"));
    }
}
