//! Cross-platform `Command` polyfill. The real `creation_flags` method
//! only exists on Windows; calling it via `if cfg!(...)` branches
//! requires the method to be in scope on *every* platform. Provide a
//! no-op shim on non-Windows so call sites compile uniformly.
//!
//! Usage:
//! ```ignore
//! use crate::commands_ext::CommandExt;
//! std::process::Command::new("x").creation_flags(0);
//! ```

#[cfg(target_os = "windows")]
pub use std::os::windows::process::CommandExt;

#[cfg(not(target_os = "windows"))]
pub trait CommandExt {
    /// No-op polyfill for Windows' `creation_flags`. The argument is
    /// ignored on platforms other than Windows.
    fn creation_flags(&mut self, _flags: u32) -> &mut Self;
}

#[cfg(not(target_os = "windows"))]
impl CommandExt for std::process::Command {
    fn creation_flags(&mut self, _flags: u32) -> &mut Self {
        self
    }
}