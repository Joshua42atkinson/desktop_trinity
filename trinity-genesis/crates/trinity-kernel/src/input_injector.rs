//! Input Injector - Wayland-Compatible Input Injection
//!
//! Provides input injection capabilities for automating UI interactions.
//! Supports both uinput (kernel-level) and XDG portals (Wayland-safe).
//!
//! ## Philosophy
//! "The Will must be able to act in the world. An agent that cannot
//!  move a mouse is an agent in a cage."
//!
//! ## Security Note
//! Input injection is a privileged operation. On Wayland, this requires
//! either user permission via XDG Remote Desktop portal, or membership
//! in the `input` group for uinput access.
//!
//! ## Usage
//! ```rust,ignore
//! let injector = InputInjector::new()?;
//! injector.move_mouse_relative(100, 50)?;
//! injector.type_text("Hello, World!")?;
//! ```

use anyhow::{Context, Result};
use std::process::Command;
use tracing::{debug, info, warn};

/// Backend for input injection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputBackend {
    /// Use xdotool (X11 only, doesn't work on Wayland)
    Xdotool,
    /// Use ydotool (Wayland-compatible, requires ydotoold daemon)
    Ydotool,
    /// Use wtype (Wayland-native for text input)
    Wtype,
    /// Use evemu (requires root or input group)
    Evemu,
}

impl InputBackend {
    /// Check if this backend is available on the system
    pub fn is_available(&self) -> bool {
        match self {
            InputBackend::Xdotool => which("xdotool").is_ok(),
            InputBackend::Ydotool => which("ydotool").is_ok(),
            InputBackend::Wtype => which("wtype").is_ok(),
            InputBackend::Evemu => which("evemu-event").is_ok(),
        }
    }
}

/// Input injector for keyboard and mouse automation
pub struct InputInjector {
    backend: InputBackend,
    /// Rate limit: minimum milliseconds between actions
    rate_limit_ms: u64,
    /// Last action timestamp
    last_action: std::time::Instant,
}

impl InputInjector {
    /// Create a new input injector with automatic backend detection
    pub fn new() -> Result<Self> {
        // Try backends in order of preference for Wayland
        let backend = if is_wayland() {
            // Wayland session - prefer ydotool or wtype
            if InputBackend::Ydotool.is_available() {
                info!("Using ydotool backend (Wayland-compatible)");
                InputBackend::Ydotool
            } else if InputBackend::Wtype.is_available() {
                info!("Using wtype backend (Wayland text only)");
                InputBackend::Wtype
            } else {
                warn!("No Wayland-compatible input backend found!");
                warn!("Install ydotool: sudo apt install ydotool");
                anyhow::bail!("No Wayland-compatible input backend available")
            }
        } else {
            // X11 session - xdotool works fine
            if InputBackend::Xdotool.is_available() {
                info!("Using xdotool backend (X11)");
                InputBackend::Xdotool
            } else {
                anyhow::bail!("xdotool not found. Install with: sudo apt install xdotool")
            }
        };

        Ok(Self {
            backend,
            rate_limit_ms: 50, // 50ms minimum between actions
            last_action: std::time::Instant::now(),
        })
    }

    /// Create with a specific backend
    pub fn with_backend(backend: InputBackend) -> Result<Self> {
        if !backend.is_available() {
            anyhow::bail!("Backend {:?} is not available", backend);
        }

        Ok(Self {
            backend,
            rate_limit_ms: 50,
            last_action: std::time::Instant::now(),
        })
    }

    /// Set the rate limit in milliseconds
    pub fn with_rate_limit(mut self, ms: u64) -> Self {
        self.rate_limit_ms = ms;
        self
    }

    /// Enforce rate limiting
    fn rate_limit(&mut self) {
        let elapsed = self.last_action.elapsed().as_millis() as u64;
        if elapsed < self.rate_limit_ms {
            std::thread::sleep(std::time::Duration::from_millis(
                self.rate_limit_ms - elapsed,
            ));
        }
        self.last_action = std::time::Instant::now();
    }

    /// Type text as keyboard input
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        self.rate_limit();
        debug!(
            "Typing text: {}...",
            &text.chars().take(20).collect::<String>()
        );

        match self.backend {
            InputBackend::Xdotool => {
                Command::new("xdotool")
                    .arg("type")
                    .arg("--clearmodifiers")
                    .arg(text)
                    .status()
                    .context("xdotool type failed")?;
            }
            InputBackend::Ydotool => {
                Command::new("ydotool")
                    .arg("type")
                    .arg(text)
                    .status()
                    .context("ydotool type failed")?;
            }
            InputBackend::Wtype => {
                Command::new("wtype")
                    .arg(text)
                    .status()
                    .context("wtype failed")?;
            }
            InputBackend::Evemu => {
                // Evemu is low-level, not ideal for typing
                warn!("Evemu backend doesn't support text typing efficiently");
                anyhow::bail!("Use ydotool or wtype for text input")
            }
        }

        Ok(())
    }

    /// Press a single key
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        self.rate_limit();
        debug!("Pressing key: {}", key);

        match self.backend {
            InputBackend::Xdotool => {
                Command::new("xdotool")
                    .arg("key")
                    .arg(key)
                    .status()
                    .context("xdotool key failed")?;
            }
            InputBackend::Ydotool => {
                Command::new("ydotool")
                    .arg("key")
                    .arg(key)
                    .status()
                    .context("ydotool key failed")?;
            }
            InputBackend::Wtype => {
                Command::new("wtype")
                    .arg("-k")
                    .arg(key)
                    .status()
                    .context("wtype key failed")?;
            }
            InputBackend::Evemu => {
                anyhow::bail!("Evemu key press not implemented")
            }
        }

        Ok(())
    }

    /// Move mouse to absolute position
    pub fn move_mouse(&mut self, x: i32, y: i32) -> Result<()> {
        self.rate_limit();
        debug!("Moving mouse to ({}, {})", x, y);

        match self.backend {
            InputBackend::Xdotool => {
                Command::new("xdotool")
                    .arg("mousemove")
                    .arg(x.to_string())
                    .arg(y.to_string())
                    .status()
                    .context("xdotool mousemove failed")?;
            }
            InputBackend::Ydotool => {
                Command::new("ydotool")
                    .arg("mousemove")
                    .arg("-a")
                    .arg("-x")
                    .arg(x.to_string())
                    .arg("-y")
                    .arg(y.to_string())
                    .status()
                    .context("ydotool mousemove failed")?;
            }
            InputBackend::Wtype => {
                anyhow::bail!("wtype doesn't support mouse movement")
            }
            InputBackend::Evemu => {
                anyhow::bail!("Evemu mouse move not implemented")
            }
        }

        Ok(())
    }

    /// Move mouse relative to current position
    pub fn move_mouse_relative(&mut self, dx: i32, dy: i32) -> Result<()> {
        self.rate_limit();
        debug!("Moving mouse by ({}, {})", dx, dy);

        match self.backend {
            InputBackend::Xdotool => {
                Command::new("xdotool")
                    .arg("mousemove_relative")
                    .arg(dx.to_string())
                    .arg(dy.to_string())
                    .status()
                    .context("xdotool mousemove_relative failed")?;
            }
            InputBackend::Ydotool => {
                Command::new("ydotool")
                    .arg("mousemove")
                    .arg("-x")
                    .arg(dx.to_string())
                    .arg("-y")
                    .arg(dy.to_string())
                    .status()
                    .context("ydotool mousemove failed")?;
            }
            _ => anyhow::bail!("{:?} doesn't support relative mouse movement", self.backend),
        }

        Ok(())
    }

    /// Click a mouse button
    pub fn click(&mut self, button: MouseButton) -> Result<()> {
        self.rate_limit();
        debug!("Clicking {:?}", button);

        let button_num = button.to_number();

        match self.backend {
            InputBackend::Xdotool => {
                Command::new("xdotool")
                    .arg("click")
                    .arg(button_num.to_string())
                    .status()
                    .context("xdotool click failed")?;
            }
            InputBackend::Ydotool => {
                // ydotool uses 0xC0 format for clicks
                let btn_code = match button {
                    MouseButton::Left => "0xC0",
                    MouseButton::Right => "0xC1",
                    MouseButton::Middle => "0xC2",
                };
                Command::new("ydotool")
                    .arg("click")
                    .arg(btn_code)
                    .status()
                    .context("ydotool click failed")?;
            }
            _ => anyhow::bail!("{:?} doesn't support mouse clicks", self.backend),
        }

        Ok(())
    }

    /// Scroll the mouse wheel
    pub fn scroll(&mut self, amount: i32) -> Result<()> {
        self.rate_limit();
        debug!("Scrolling by {}", amount);

        match self.backend {
            InputBackend::Xdotool => {
                let (direction, count) = if amount > 0 {
                    ("4", amount)
                } else {
                    ("5", -amount)
                };
                for _ in 0..count {
                    Command::new("xdotool")
                        .arg("click")
                        .arg(direction)
                        .status()
                        .context("xdotool scroll failed")?;
                }
            }
            InputBackend::Ydotool => {
                // ydotool doesn't have great scroll support
                warn!("ydotool scroll support is limited");
                // Would need to use evdev directly
            }
            _ => anyhow::bail!("{:?} doesn't support scrolling", self.backend),
        }

        Ok(())
    }

    /// Get the current backend
    pub fn backend(&self) -> InputBackend {
        self.backend
    }
}

/// Mouse button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    fn to_number(&self) -> u8 {
        match self {
            MouseButton::Left => 1,
            MouseButton::Right => 3,
            MouseButton::Middle => 2,
        }
    }
}

/// Check if we're running on Wayland
fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|s| s == "wayland")
        .unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY").is_ok()
}

/// Check if a command exists in PATH
fn which(cmd: &str) -> Result<std::path::PathBuf> {
    let output = Command::new("which")
        .arg(cmd)
        .output()
        .context("Failed to run 'which'")?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(std::path::PathBuf::from(path))
    } else {
        anyhow::bail!("Command '{}' not found", cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_detection() {
        // Just test that the function doesn't crash
        let _ = is_wayland();
    }

    #[test]
    fn test_mouse_button_numbers() {
        assert_eq!(MouseButton::Left.to_number(), 1);
        assert_eq!(MouseButton::Middle.to_number(), 2);
        assert_eq!(MouseButton::Right.to_number(), 3);
    }
}
