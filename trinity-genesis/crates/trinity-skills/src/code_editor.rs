use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

/// Code Editor Skill
///
/// Provides capabilities to read/write files and execute commands.
/// Designed for headless usage in Trinity Brain.
pub struct CodeEditor;

impl CodeEditor {
    /// Read a file from the filesystem
    pub async fn read_file(path: impl AsRef<Path>) -> Result<String> {
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }

    /// Write content to a file (overwriting)
    pub async fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, content).await?;
        Ok(())
    }

    /// Run a shell command
    pub async fn run_command(command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(command)
            .args(args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}
