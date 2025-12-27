//! Symbol Registry - Naming Consistency System
//!
//! Prevents AI agents from recreating existing code by forcing lookup-before-create.
//! Indexes all symbols in the workspace and provides fuzzy search with synonyms.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================================
// Symbol Types
// ============================================================================

/// Kind of symbol being tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Rust function or method
    Function,
    /// Struct or class
    Struct,
    /// Enum type
    Enum,
    /// Trait or interface
    Trait,
    /// Module or namespace
    Module,
    /// Constant or static
    Constant,
    /// File path
    File,
    /// Tool name
    Tool,
}

impl SymbolKind {
    /// Get the naming convention for this kind
    pub fn convention(&self) -> NamingConvention {
        match self {
            SymbolKind::Function => NamingConvention::SnakeCase,
            SymbolKind::Struct => NamingConvention::PascalCase,
            SymbolKind::Enum => NamingConvention::PascalCase,
            SymbolKind::Trait => NamingConvention::PascalCase,
            SymbolKind::Module => NamingConvention::SnakeCase,
            SymbolKind::Constant => NamingConvention::ScreamingSnakeCase,
            SymbolKind::File => NamingConvention::SnakeCase,
            SymbolKind::Tool => NamingConvention::SnakeCase,
        }
    }
}

/// Naming convention style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingConvention {
    /// snake_case
    SnakeCase,
    /// PascalCase
    PascalCase,
    /// camelCase
    CamelCase,
    /// SCREAMING_SNAKE_CASE
    ScreamingSnakeCase,
}

impl NamingConvention {
    /// Convert a phrase to this naming convention
    pub fn apply(&self, phrase: &str) -> String {
        let words = Self::split_into_words(phrase);

        match self {
            NamingConvention::SnakeCase => words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("_"),
            NamingConvention::PascalCase => words
                .iter()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c
                            .to_uppercase()
                            .chain(chars.map(|c| c.to_ascii_lowercase()))
                            .collect(),
                    }
                })
                .collect(),
            NamingConvention::CamelCase => {
                let mut result = String::new();
                for (i, word) in words.iter().enumerate() {
                    if i == 0 {
                        result.push_str(&word.to_lowercase());
                    } else {
                        let mut chars = word.chars();
                        if let Some(c) = chars.next() {
                            result.push(c.to_ascii_uppercase());
                            result.extend(chars.map(|c| c.to_ascii_lowercase()));
                        }
                    }
                }
                result
            }
            NamingConvention::ScreamingSnakeCase => words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
        }
    }

    /// Split a phrase into words
    fn split_into_words(phrase: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();

        for c in phrase.chars() {
            if c == '_' || c == '-' || c == ' ' {
                if !current_word.is_empty() {
                    words.push(current_word);
                    current_word = String::new();
                }
            } else if c.is_uppercase()
                && !current_word.is_empty()
                && !current_word
                    .chars()
                    .last()
                    .map(|lc| lc.is_uppercase())
                    .unwrap_or(false)
            {
                // Start of new word in camelCase/PascalCase
                words.push(current_word);
                current_word = c.to_string();
            } else {
                current_word.push(c);
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
    }
}

// ============================================================================
// Symbol Info
// ============================================================================

/// Information about a symbol in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Canonical name of the symbol
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// File path where defined
    pub path: PathBuf,
    /// Line number (1-indexed)
    pub line: Option<u32>,
    /// Brief description or doc summary
    pub doc_summary: Option<String>,
    /// How often this symbol is referenced
    pub usage_count: u32,
    /// Keywords extracted from name and docs
    pub keywords: Vec<String>,
}

impl SymbolInfo {
    pub fn new(name: impl Into<String>, kind: SymbolKind, path: PathBuf) -> Self {
        let name = name.into();
        let keywords = Self::extract_keywords(&name);

        Self {
            name,
            kind,
            path,
            line: None,
            doc_summary: None,
            usage_count: 0,
            keywords,
        }
    }

    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        let doc = doc.into();
        // Add doc words to keywords
        self.keywords.extend(Self::extract_keywords(&doc));
        self.doc_summary = Some(doc);
        self
    }

    /// Extract keywords from a string
    fn extract_keywords(s: &str) -> Vec<String> {
        NamingConvention::split_into_words(s)
            .into_iter()
            .filter(|w| w.len() > 2)
            .map(|w| w.to_lowercase())
            .collect()
    }
}

// ============================================================================
// Symbol Match
// ============================================================================

/// A match result from symbol search
#[derive(Debug, Clone)]
pub struct SymbolMatch {
    /// The matched symbol
    pub symbol: SymbolInfo,
    /// Match score (higher = better match)
    pub score: f32,
    /// Reason for the match
    pub reason: String,
}

// ============================================================================
// Synonym Groups
// ============================================================================

/// Built-in synonym groups for common programming concepts
pub struct Synonyms {
    groups: Vec<Vec<&'static str>>,
}

impl Default for Synonyms {
    fn default() -> Self {
        Self {
            groups: vec![
                vec![
                    "create",
                    "new",
                    "spawn",
                    "init",
                    "make",
                    "build",
                    "construct",
                ],
                vec![
                    "delete", "remove", "destroy", "kill", "drop", "cleanup", "dispose",
                ],
                vec![
                    "get", "fetch", "retrieve", "load", "read", "query", "find", "lookup",
                ],
                vec![
                    "set", "update", "modify", "edit", "change", "patch", "write",
                ],
                vec![
                    "send",
                    "emit",
                    "dispatch",
                    "publish",
                    "broadcast",
                    "notify",
                    "push",
                ],
                vec!["receive", "handle", "process", "execute", "run", "perform"],
                vec!["check", "validate", "verify", "test", "assert", "ensure"],
                vec!["start", "begin", "open", "launch", "activate", "enable"],
                vec![
                    "stop",
                    "end",
                    "close",
                    "shutdown",
                    "deactivate",
                    "disable",
                    "halt",
                ],
                vec!["list", "enumerate", "iterate", "scan", "walk", "traverse"],
                vec!["parse", "decode", "deserialize", "extract", "interpret"],
                vec!["format", "encode", "serialize", "render", "stringify"],
                vec!["config", "settings", "options", "preferences", "params"],
                vec!["error", "fault", "failure", "exception", "issue", "problem"],
                vec!["message", "msg", "text", "content", "payload", "data"],
                vec!["response", "reply", "result", "output", "return"],
                vec!["request", "query", "input", "command", "instruction"],
            ],
        }
    }
}

impl Synonyms {
    /// Find all synonyms for a word
    pub fn get_synonyms(&self, word: &str) -> Vec<&'static str> {
        let word_lower = word.to_lowercase();

        for group in &self.groups {
            if group.iter().any(|w| w.to_lowercase() == word_lower) {
                return group.to_vec();
            }
        }

        Vec::new()
    }

    /// Check if two words are synonyms
    pub fn are_synonyms(&self, a: &str, b: &str) -> bool {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower == b_lower {
            return true;
        }

        for group in &self.groups {
            let has_a = group.iter().any(|w| w.to_lowercase() == a_lower);
            let has_b = group.iter().any(|w| w.to_lowercase() == b_lower);
            if has_a && has_b {
                return true;
            }
        }

        false
    }
}

// ============================================================================
// Symbol Registry
// ============================================================================

/// Registry of all symbols in the codebase
pub struct SymbolRegistry {
    /// All indexed symbols, keyed by name
    symbols: HashMap<String, SymbolInfo>,
    /// Index by kind
    by_kind: HashMap<SymbolKind, Vec<String>>,
    /// Index by file
    by_file: HashMap<PathBuf, Vec<String>>,
    /// Keyword index for fuzzy search
    keyword_index: HashMap<String, Vec<String>>,
    /// Synonym dictionary
    synonyms: Synonyms,
}

impl SymbolRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            by_kind: HashMap::new(),
            by_file: HashMap::new(),
            keyword_index: HashMap::new(),
            synonyms: Synonyms::default(),
        }
    }

    /// Register a new symbol
    pub fn register(&mut self, info: SymbolInfo) {
        let name = info.name.clone();

        // Add to kind index
        self.by_kind
            .entry(info.kind)
            .or_default()
            .push(name.clone());

        // Add to file index
        self.by_file
            .entry(info.path.clone())
            .or_default()
            .push(name.clone());

        // Add to keyword index
        for keyword in &info.keywords {
            self.keyword_index
                .entry(keyword.clone())
                .or_default()
                .push(name.clone());
        }

        // Also index synonyms
        for keyword in &info.keywords {
            for syn in self.synonyms.get_synonyms(keyword) {
                self.keyword_index
                    .entry(syn.to_string())
                    .or_default()
                    .push(name.clone());
            }
        }

        self.symbols.insert(name, info);
    }

    /// Find existing symbols matching an intent
    ///
    /// This is the key method - MUST be called before creating new symbols
    pub fn find_existing(&self, intent: &str) -> Vec<SymbolMatch> {
        let search_keywords: Vec<String> = NamingConvention::split_into_words(intent)
            .into_iter()
            .map(|w| w.to_lowercase())
            .collect();

        if search_keywords.is_empty() {
            return Vec::new();
        }

        let mut scores: HashMap<String, (f32, Vec<String>)> = HashMap::new();

        // Score each symbol based on keyword matches
        for keyword in &search_keywords {
            // Direct matches
            if let Some(matches) = self.keyword_index.get(keyword) {
                for name in matches {
                    let entry = scores.entry(name.clone()).or_insert((0.0, Vec::new()));
                    entry.0 += 1.0;
                    entry.1.push(format!("matches '{}'", keyword));
                }
            }

            // Synonym matches (slightly lower score)
            for syn in self.synonyms.get_synonyms(keyword) {
                if let Some(matches) = self.keyword_index.get(syn) {
                    for name in matches {
                        let entry = scores.entry(name.clone()).or_insert((0.0, Vec::new()));
                        entry.0 += 0.7;
                        entry.1.push(format!("synonym '{}' ~ '{}'", keyword, syn));
                    }
                }
            }
        }

        // Convert to matches
        let mut matches: Vec<SymbolMatch> = scores
            .into_iter()
            .filter(|(_, (score, _))| *score >= 1.0)
            .filter_map(|(name, (score, reasons))| {
                self.symbols.get(&name).map(|symbol| SymbolMatch {
                    symbol: symbol.clone(),
                    score,
                    reason: reasons.join(", "),
                })
            })
            .collect();

        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top 10
        matches.truncate(10);
        matches
    }

    /// Suggest a canonical name for a new symbol
    pub fn suggest_name(&self, kind: SymbolKind, description: &str) -> String {
        let convention = kind.convention();
        convention.apply(description)
    }

    /// Get a symbol by name
    pub fn get(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbols.get(name)
    }

    /// List all symbols of a kind
    pub fn list_by_kind(&self, kind: SymbolKind) -> Vec<&SymbolInfo> {
        self.by_kind
            .get(&kind)
            .map(|names| names.iter().filter_map(|n| self.symbols.get(n)).collect())
            .unwrap_or_default()
    }

    /// List all symbols in a file
    pub fn list_by_file(&self, path: &Path) -> Vec<&SymbolInfo> {
        self.by_file
            .get(path)
            .map(|names| names.iter().filter_map(|n| self.symbols.get(n)).collect())
            .unwrap_or_default()
    }

    /// Index a Rust source file
    pub fn index_rust_file(&mut self, path: &Path) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let mut count = 0;

        // Simple regex-based parsing for common Rust constructs
        // In production, you'd want to use tree-sitter or syn

        // Functions: fn name(
        let fn_re = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(")?;
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = fn_re.captures(line) {
                if let Some(name) = cap.get(1) {
                    self.register(
                        SymbolInfo::new(name.as_str(), SymbolKind::Function, path.to_path_buf())
                            .with_line((line_num + 1) as u32),
                    );
                    count += 1;
                }
            }
        }

        // Structs: struct Name
        let struct_re = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)")?;
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = struct_re.captures(line) {
                if let Some(name) = cap.get(1) {
                    self.register(
                        SymbolInfo::new(name.as_str(), SymbolKind::Struct, path.to_path_buf())
                            .with_line((line_num + 1) as u32),
                    );
                    count += 1;
                }
            }
        }

        // Enums: enum Name
        let enum_re = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?enum\s+(\w+)")?;
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = enum_re.captures(line) {
                if let Some(name) = cap.get(1) {
                    self.register(
                        SymbolInfo::new(name.as_str(), SymbolKind::Enum, path.to_path_buf())
                            .with_line((line_num + 1) as u32),
                    );
                    count += 1;
                }
            }
        }

        // Traits: trait Name
        let trait_re = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?trait\s+(\w+)")?;
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = trait_re.captures(line) {
                if let Some(name) = cap.get(1) {
                    self.register(
                        SymbolInfo::new(name.as_str(), SymbolKind::Trait, path.to_path_buf())
                            .with_line((line_num + 1) as u32),
                    );
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Index an entire workspace
    pub fn index_workspace(&mut self, root: &Path) -> Result<usize> {
        let mut total = 0;

        // Walk directory recursively
        fn walk_dir(registry: &mut SymbolRegistry, dir: &Path, total: &mut usize) -> Result<()> {
            if !dir.is_dir() {
                return Ok(());
            }

            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip hidden and target directories
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }

                if path.is_dir() {
                    walk_dir(registry, &path, total)?;
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "rs" {
                        match registry.index_rust_file(&path) {
                            Ok(count) => *total += count,
                            Err(e) => tracing::warn!("Failed to index {:?}: {}", path, e),
                        }
                    }
                }
            }

            Ok(())
        }

        walk_dir(self, root, &mut total)?;

        tracing::info!("Indexed {} symbols from {:?}", total, root);
        Ok(total)
    }

    /// Get total symbol count
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

impl Default for SymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naming_conventions() {
        assert_eq!(
            NamingConvention::SnakeCase.apply("createNewModel"),
            "create_new_model"
        );
        assert_eq!(
            NamingConvention::PascalCase.apply("create_new_model"),
            "CreateNewModel"
        );
        assert_eq!(
            NamingConvention::CamelCase.apply("create_new_model"),
            "createNewModel"
        );
        assert_eq!(
            NamingConvention::ScreamingSnakeCase.apply("max_tokens"),
            "MAX_TOKENS"
        );
    }

    #[test]
    fn test_synonyms() {
        let syns = Synonyms::default();

        assert!(syns.are_synonyms("create", "new"));
        assert!(syns.are_synonyms("delete", "remove"));
        assert!(syns.are_synonyms("get", "fetch"));
        assert!(!syns.are_synonyms("create", "delete"));
    }

    #[test]
    fn test_symbol_search() {
        let mut registry = SymbolRegistry::new();

        registry.register(SymbolInfo::new(
            "create_model",
            SymbolKind::Function,
            PathBuf::from("test.rs"),
        ));
        registry.register(SymbolInfo::new(
            "spawn_agent",
            SymbolKind::Function,
            PathBuf::from("test.rs"),
        ));

        // Should find create_model when searching for "new model" (synonyms)
        let matches = registry.find_existing("new model");
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.symbol.name == "create_model"));

        // Should find spawn_agent when searching for "create agent"
        let matches = registry.find_existing("create agent");
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.symbol.name == "spawn_agent"));
    }

    #[test]
    fn test_suggest_name() {
        let registry = SymbolRegistry::new();

        assert_eq!(
            registry.suggest_name(SymbolKind::Function, "create new model"),
            "create_new_model"
        );
        assert_eq!(
            registry.suggest_name(SymbolKind::Struct, "brain resource"),
            "BrainResource"
        );
    }
}
