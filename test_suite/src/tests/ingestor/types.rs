//! Test data types for ingestor tests.

use crate::test_environment::TestEnvironment;

/// All supported file types to test
#[derive(Debug, Clone)]
pub struct FileTypeTest {
    pub file_path: String,
    pub file_type: &'static str,
    pub extension: &'static str,
    #[allow(dead_code)]
    pub should_succeed: bool,
}

impl FileTypeTest {
    pub fn new(
        relative_path: &str,
        file_type: &'static str,
        extension: &'static str,
        should_succeed: bool,
    ) -> Self {
        Self {
            file_path: relative_path.to_string(),
            file_type,
            extension,
            should_succeed,
        }
    }
}

/// Get all file type tests to run
pub fn get_all_file_type_tests(env: &TestEnvironment) -> Vec<FileTypeTest> {
    vec![
        // Standard text files
        FileTypeTest::new("readme.txt", "text", "txt", true),
        FileTypeTest::new("sample.md", "markdown", "md", true),
        FileTypeTest::new("sample.rst", "rst", "rst", true),
        FileTypeTest::new("sample.log", "log", "log", true),
        FileTypeTest::new("sample.xml", "xml", "xml", true),
        FileTypeTest::new("sample.html", "html", "html", true),
        // Code files
        FileTypeTest::new("code_samples/sample.rs", "rust", "rs", true),
        FileTypeTest::new("code_samples/sample.py", "python", "py", true),
        FileTypeTest::new("code_samples/sample.js", "javascript", "js", true),
        FileTypeTest::new("code_samples/sample.ts", "typescript", "ts", true),
        // Config files
        FileTypeTest::new("config_files/app.yaml", "yaml", "yaml", true),
        FileTypeTest::new("config_files/settings.ini", "ini", "ini", true),
        FileTypeTest::new("config_files/config.toml", "toml", "toml", true),
        FileTypeTest::new("config_files/data.csv", "csv", "csv", true),
        // Scripts
        FileTypeTest::new("code_samples/script.sh", "shell", "sh", true),
        FileTypeTest::new("code_samples/query.sql", "sql", "sql", true),
        // Subtitles
        FileTypeTest::new("sample.srt", "subtitle", "srt", true),
        // Image (SVG - metadata extraction)
        FileTypeTest::new("sample.svg", "image/svg", "svg", true),
        // JSON/JSONL (special handling)
        FileTypeTest::new("config_files/data.json", "json", "json", true),
        FileTypeTest::new("config_files/data.jsonl", "jsonl", "jsonl", true),
    ]
}
