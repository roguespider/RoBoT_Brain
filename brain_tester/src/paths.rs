//! Runtime path resolution for the brain_tester suite.
//!
//! The robot_brain project layout is fixed: `brain_tester/` lives one directory
//! below the project root, and the compiled server binary lands in
//! `<root>/target/release/robot_brain`. These helpers resolve that layout
//! relative to the current working directory at runtime instead of baking a
//! compile-time absolute path into the binary.

use std::path::{Path, PathBuf};

/// The server binary name for the host platform.
fn binary_name() -> &'static str {
    if cfg!(windows) {
        "robot_brain.exe"
    } else {
        "robot_brain"
    }
}

/// Candidate project-root locations, evaluated by walking up from the current
/// working directory to its ancestors. This handles being run from any
/// subdirectory (e.g. `brain_tester/target/release/`), not just `brain_tester/`.
fn root_candidates() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut candidates = Vec::new();
    let mut current: Option<&Path> = Some(cwd.as_path());
    while let Some(dir) = current {
        candidates.push(dir.to_path_buf());
        current = dir.parent();
    }
    candidates
}

/// True when a directory looks like the robot_brain project root: it either
/// already contains a built server binary or a `Cargo.toml` declaring the
/// `robot_brain` package.
fn is_robot_brain_root(dir: &Path) -> bool {
    let bin = dir.join("target/release").join(binary_name());
    if bin.exists() {
        return true;
    }
    matches!(read_package_name(&dir.join("Cargo.toml")), Some(name) if name == "robot_brain")
}

/// Extract the `name = "..."` value from a Cargo.toml, if readable.
fn read_package_name(cargo_toml: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(cargo_toml).ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("name") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(value) = rest.strip_prefix('"') else {
            continue;
        };
        return Some(value.split('"').next().unwrap_or("").to_string());
    }
    None
}

/// Resolve the robot_brain project root directory at runtime.
///
/// Walks up from the current working directory to the filesystem root and
/// returns the first ancestor that looks like the project root. Falls back to
/// `cwd` so `build_server` can still construct a path if no root is detected.
pub fn project_root() -> PathBuf {
    for candidate in root_candidates() {
        if is_robot_brain_root(&candidate) {
            return candidate;
        }
    }
    // Fallback: assume the current working directory is within the project.
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolve the directory the test suite itself lives in.
///
/// This is the project root's `brain_tester` subdirectory when present,
/// otherwise the current working directory.
pub fn test_suite_dir() -> PathBuf {
    let root = project_root();
    let candidate = root.join("brain_tester");
    if candidate.is_dir() {
        return candidate;
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolve the expected path to the compiled server binary, if it exists.
pub fn server_binary() -> Option<PathBuf> {
    let bin = project_root().join("target/release").join(binary_name());
    if bin.exists() {
        return Some(bin);
    }
    None
}
