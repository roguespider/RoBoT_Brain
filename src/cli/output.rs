// src/cli/output.rs
//! Output formatting utilities

use std::fmt;

/// Colored output
pub mod color {
    use std::fmt;

    pub struct Green<T>(pub T);
    pub struct Red<T>(pub T);
    pub struct Yellow<T>(pub T);
    pub struct Cyan<T>(pub T);
    pub struct Bold<T>(pub T);

    impl<T: fmt::Display> fmt::Display for Green<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[32m{}\x1b[0m", self.0)
        }
    }

    impl<T: fmt::Display> fmt::Display for Red<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[31m{}\x1b[0m", self.0)
        }
    }

    impl<T: fmt::Display> fmt::Display for Yellow<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[33m{}\x1b[0m", self.0)
        }
    }

    impl<T: fmt::Display> fmt::Display for Cyan<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[36m{}\x1b[0m", self.0)
        }
    }

    impl<T: fmt::Display> fmt::Display for Bold<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[1m{}\x1b[0m", self.0)
        }
    }
}

/// Separator types for tables
pub enum Separator {
    Line,
    Double,
    Dots,
}

impl fmt::Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Separator::Line => write!(f, "─────────────────────────────────────────"),
            Separator::Double => write!(f, "═════════════════════════════════════════"),
            Separator::Dots => write!(f, "· · · · · · · · · · · · · · · · · · · ·"),
        }
    }
}

/// Print success message
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        println!("\x1b[32m✓\x1b[0m {}", format!($($arg)*))
    };
}

/// Print error message
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("\x1b[31m✗\x1b[0m {}", format!($($arg)*))
    };
}

/// Print info message
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("\x1b[36mℹ\x1b[0m {}", format!($($arg)*))
    };
}

/// Print warning message
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("\x1b[33m⚠\x1b[0m {}", format!($($arg)*))
    };
}

/// Print a table row
pub fn table_row(columns: &[&str], widths: &[usize]) {
    let mut row = String::new();
    for (col, &width) in columns.iter().zip(widths.iter()) {
        row.push_str(&format!("{:width$}  ", col, width = width));
    }
    println!("{}", row.trim_end());
}

/// Print table headers
pub fn table_header(columns: &[&str], widths: &[usize]) {
    table_row(columns, widths);
    let sep: String = widths.iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}", sep);
}
