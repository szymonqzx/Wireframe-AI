//! Security validation and sanitization for the adapter.
//!
//! This module provides security utilities for:
//! - Shell command validation and allowlisting
//! - Path validation and traversal prevention
//! - String sanitization

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Allowed shell commands for security.
/// This prevents command injection by only allowing specific safe commands.
pub const ALLOWED_COMMANDS: &[&str] = &[
    "ls", "dir", "cd", "pwd", "echo", "cat", "type", "head", "tail", "grep", "find", "git",
    "cargo", "npm", "pip", "python", "python3", "node", "rustc", "clang", "gcc", "mkdir", "rmdir",
    "rm", "del", "cp", "copy", "mv", "move", "touch", "file", "stat", "wc", "sort", "uniq", "cut",
    "awk", "sed",
];

/// Validate and sanitize a shell command.
/// Returns an error if the command is not in the allowlist or contains dangerous patterns.
pub fn validate_shell_command(command: &str) -> Result<String> {
    // Extract the base command (first word)
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    let base_cmd = parts[0];

    // Check if command is in allowlist
    if !ALLOWED_COMMANDS.contains(&base_cmd) {
        return Err(anyhow::anyhow!(
            "Command '{}' is not in the allowlist. Allowed commands: {:?}",
            base_cmd,
            ALLOWED_COMMANDS
        ));
    }

    // Check for dangerous patterns (pipe redirection, command chaining, etc.)
    let dangerous_patterns = ["|", "&", "&&", "||", ";", "$(", "`", "\n", "\r"];
    for pattern in &dangerous_patterns {
        if command.contains(pattern) {
            return Err(anyhow::anyhow!(
                "Command contains dangerous pattern '{}': {}",
                pattern,
                command
            ));
        }
    }

    // Validate arguments (no shell metacharacters)
    for arg in &parts[1..] {
        // Check for shell metacharacters in arguments
        let metachars = [
            "$", "`", "\\", "\"", "'", "<", ">", "*", "?", "[", "]", "{", "}",
        ];
        for meta in &metachars {
            if arg.contains(meta) {
                return Err(anyhow::anyhow!(
                    "Argument contains shell metacharacter '{}': {}",
                    meta,
                    arg
                ));
            }
        }
    }

    Ok(command.to_string())
}

/// Validate that a path is safe and doesn't contain path traversal attempts.
/// Returns the canonical path if safe, or an error if unsafe.
pub fn validate_path(path: &str, allowed_base: Option<&Path>) -> Result<PathBuf> {
    let path_obj = Path::new(path);

    // Check for obvious path traversal patterns
    let path_str = path.to_string();
    if path_str.contains("..") || path_str.contains("~") {
        return Err(anyhow::anyhow!(
            "Path contains traversal or home directory expansion: {}",
            path_str
        ));
    }

    // Resolve to canonical path
    let canonical = path_obj
        .canonicalize()
        .context(format!("Failed to resolve path: {}", path_str))?;

    // If an allowed base is provided, ensure the path is within it
    if let Some(base) = allowed_base {
        let canonical_base = base
            .canonicalize()
            .context("Failed to resolve allowed base directory")?;

        if !canonical.starts_with(&canonical_base) {
            return Err(anyhow::anyhow!(
                "Path '{}' is outside allowed directory '{}'",
                canonical.display(),
                canonical_base.display()
            ));
        }
    }

    Ok(canonical)
}

/// Validate a path for write operations.
/// Unlike `validate_path`, this allows the file itself to not exist yet.
/// It validates the parent directory exists and is within the allowed base.
pub fn validate_path_for_write(path: &str, allowed_base: Option<&Path>) -> Result<PathBuf> {
    let path_obj = Path::new(path);

    // Check for obvious path traversal patterns
    let path_str = path.to_string();
    if path_str.contains("..") || path_str.contains("~") {
        return Err(anyhow::anyhow!(
            "Path contains traversal or home directory expansion: {}",
            path_str
        ));
    }

    // Resolve to absolute path
    let absolute = if path_obj.is_absolute() {
        path_obj.to_path_buf()
    } else {
        std::env::current_dir()
            .context("Failed to get current directory")?
            .join(path_obj)
    };

    // Validate parent directory exists and is within bounds
    let parent = absolute
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path has no parent directory: {}", path_str))?;

    let canonical_parent = parent.canonicalize().context(format!(
        "Failed to resolve parent directory: {}",
        parent.display()
    ))?;

    if let Some(base) = allowed_base {
        let canonical_base = base
            .canonicalize()
            .context("Failed to resolve allowed base directory")?;

        if !canonical_parent.starts_with(&canonical_base) {
            return Err(anyhow::anyhow!(
                "Path '{}' is outside allowed directory '{}'",
                absolute.display(),
                canonical_base.display()
            ));
        }
    }

    Ok(absolute)
}

/// Sanitize a string by removing null bytes and other dangerous characters.
/// Returns Cow<str> to avoid allocation when no sanitization is needed.
pub fn sanitize_string(input: &str) -> std::borrow::Cow<'_, str> {
    // Fast path: check if sanitization is needed
    let needs_sanitization = input.chars().any(|c| c == '\0' || c.is_control());

    if !needs_sanitization {
        return std::borrow::Cow::Borrowed(input);
    }

    // Slow path: allocate and sanitize
    std::borrow::Cow::Owned(
        input
            .chars()
            .filter(|c| *c != '\0' && !c.is_control())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_shell_command_allowed() {
        assert!(validate_shell_command("ls").is_ok());
        assert!(validate_shell_command("cargo build").is_ok());
        assert!(validate_shell_command("git status").is_ok());
        assert!(validate_shell_command("python script.py").is_ok());
    }

    #[test]
    fn test_validate_shell_command_empty() {
        assert!(validate_shell_command("").is_err());
        assert!(validate_shell_command("   ").is_err());
    }

    #[test]
    fn test_validate_shell_command_not_allowed() {
        // These base commands are NOT in the allowlist
        assert!(validate_shell_command("bash -c echo hello").is_err());
        assert!(validate_shell_command("curl https://example.com").is_err());
        assert!(validate_shell_command("wget http://example.com").is_err());
        assert!(validate_shell_command("nc -lvp 1234").is_err());
        assert!(validate_shell_command("ssh user@host").is_err());
    }

    #[test]
    fn test_validate_shell_command_dangerous_patterns() {
        assert!(validate_shell_command("ls | cat").is_err());
        assert!(validate_shell_command("ls && cat").is_err());
        assert!(validate_shell_command("ls || cat").is_err());
        assert!(validate_shell_command("ls; cat").is_err());
        assert!(validate_shell_command("echo $(whoami)").is_err());
        assert!(validate_shell_command("echo `whoami`").is_err());
        assert!(validate_shell_command("ls\ncat").is_err());
    }

    #[test]
    fn test_validate_shell_command_metacharacters_in_args() {
        assert!(validate_shell_command("echo $HOME").is_err());
        assert!(validate_shell_command("echo hello`world`").is_err());
        assert!(validate_shell_command("echo hello>file").is_err());
        assert!(validate_shell_command("echo hello<file").is_err());
        assert!(validate_shell_command("echo *").is_err());
        assert!(validate_shell_command("echo hello\"world\"").is_err());
    }

    #[test]
    fn test_sanitize_string_normal() {
        assert_eq!(sanitize_string("hello world"), "hello world");
        assert_eq!(sanitize_string("Hello, World! 123"), "Hello, World! 123");
    }

    #[test]
    fn test_sanitize_string_null_bytes() {
        assert_eq!(sanitize_string("hello\0world"), "helloworld");
        assert_eq!(sanitize_string("\0\0\0"), "");
    }

    #[test]
    fn test_sanitize_string_control_chars() {
        assert_eq!(sanitize_string("hello\x01\x02world"), "helloworld");
        assert_eq!(sanitize_string("hello\x1fworld"), "helloworld");
        assert_eq!(sanitize_string("hello\x7fworld"), "helloworld");
    }

    #[test]
    fn test_sanitize_string_mixed() {
        let input = "Hello\0\x01World\x02!";
        assert_eq!(sanitize_string(input), "HelloWorld!");
    }

    #[test]
    fn test_sanitize_string_cow_borrowed() {
        // Test that clean strings return Cow::Borrowed (no allocation)
        let result = sanitize_string("hello world");
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_sanitize_string_cow_owned() {
        // Test that dirty strings return Cow::Owned (allocation)
        let result = sanitize_string("hello\0world");
        assert!(matches!(result, std::borrow::Cow::Owned(_)));
    }
}
