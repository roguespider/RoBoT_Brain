//! Test data types for ingestor tests.

/// All supported file types to test
#[derive(Debug, Clone)]
pub struct FileTypeTest {
    pub file_path: String,
    pub file_type: &'static str,
    pub extension: &'static str,
}

impl FileTypeTest {
    pub fn new(
        relative_path: &str,
        file_type: &'static str,
        extension: &'static str,
    ) -> Self {
        Self {
            file_path: relative_path.to_string(),
            file_type,
            extension,
        }
    }
}

/// Get all file type tests to run
pub fn get_all_file_type_tests() -> Vec<FileTypeTest> {
    vec![
        // Standard text files
        FileTypeTest::new("readme.txt", "text", "txt"),
        FileTypeTest::new("sample.md", "markdown", "md"),
        FileTypeTest::new("sample.rst", "rst", "rst"),
        FileTypeTest::new("sample.log", "log", "log"),
        FileTypeTest::new("sample.xml", "xml", "xml"),
        FileTypeTest::new("sample.html", "html", "html"),
        // Code files
        FileTypeTest::new("code_samples/sample.rs", "rust", "rs"),
        FileTypeTest::new("code_samples/sample.py", "python", "py"),
        FileTypeTest::new("code_samples/sample.js", "javascript", "js"),
        FileTypeTest::new("code_samples/sample.ts", "typescript", "ts"),
        // Config files
        FileTypeTest::new("config_files/app.yaml", "yaml", "yaml"),
        FileTypeTest::new("config_files/settings.ini", "ini", "ini"),
        FileTypeTest::new("config_files/config.toml", "toml", "toml"),
        FileTypeTest::new("config_files/data.csv", "csv", "csv"),
        // Scripts
        FileTypeTest::new("code_samples/script.sh", "shell", "sh"),
        FileTypeTest::new("code_samples/query.sql", "sql", "sql"),
        // Subtitles
        FileTypeTest::new("sample.srt", "subtitle", "srt"),
        // Image (SVG - metadata extraction)
        FileTypeTest::new("sample.svg", "image/svg", "svg"),
        // JSON/JSONL (special handling)
        FileTypeTest::new("config_files/data.json", "json", "json"),
        FileTypeTest::new("config_files/data.jsonl", "jsonl", "jsonl"),
    ]
}
