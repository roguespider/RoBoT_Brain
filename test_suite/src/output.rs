//! Shared output module for writing to both stdout and file
//!
//! This module provides the TeeWriter and teeprintln macro for
//! outputting to both stdout and the test_suite_output.txt file.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// Global tee writer for outputting to both stdout and file
pub static TEE: Mutex<Option<TeeWriter>> = Mutex::new(None);

/// TeeWriter - writes to both stdout and a file
pub struct TeeWriter {
    file: BufWriter<File>,
}

impl TeeWriter {
    /// Create a new TeeWriter at the specified path
    pub fn new(path: &PathBuf) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file: BufWriter::new(file),
        })
    }

    /// Write a string to the file
    pub fn write(&mut self, s: &str) {
        // Write to file
        let _ = self.file.write_all(s.as_bytes());
        let _ = self.file.flush();
    }

    /// Write a string with newline to the file
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.write("\n");
    }

    /// Flush the buffer to ensure all data is written
    pub fn flush(&mut self) {
        let _ = self.file.flush();
    }
}

/// Initialize the global tee writer
pub fn init(path: &PathBuf) -> std::io::Result<()> {
    let mut tee = TEE
        .lock()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    *tee = Some(TeeWriter::new(path)?);
    Ok(())
}

/// Flush the global tee writer
pub fn flush() {
    if let Ok(mut tee) = TEE.lock() {
        if let Some(ref mut writer) = *tee {
            writer.flush();
        }
    }
}

/// Print and write to file simultaneously
#[macro_export]
macro_rules! teeprintln {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        // Print to stdout
        println!("{}", s);
        // Write to file
        if let Ok(mut tee) = $crate::output::TEE.lock() {
            if let Some(ref mut writer) = *tee {
                writer.writeln(&s);
            }
        }
    }};
}
