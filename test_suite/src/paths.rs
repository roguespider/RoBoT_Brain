//! Runtime path resolution for the test suite.
//!
//! The robot_brain project layout is fixed: `test_suite/` lives one directory
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

/// Candidate project-root locations, evaluated relative to the current
/// working directory. The test suite is normally run from `test_suite/`, so
/// `..` is the expected root; `.` covers running directly from the root.
fn root_candidates() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    vec![cwd.join(".."), cwd.clone()]
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
/// Returns the first candidate directory that looks like the project root,
/// falling back to `cwd/..` (the standard test_suite layout) so that
/// `build_server` can still construct the binary if it has not been built yet.
pub fn project_root() -> PathBuf {
    for candidate in root_candidates() {
        if is_robot_brain_root(&candidate) {
            return candidate;
        }
    }
    // Fallback: assume the canonical layout (run from test_suite/).
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("..")
}

/// Resolve the directory the test suite itself lives in.
///
/// This is the project root's `test_suite` subdirectory when present,
/// otherwise the current working directory.
pub fn test_suite_dir() -> PathBuf {
    let root = project_root();
    let candidate = root.join("test_suite");
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
