// /src/experience/reflection/pattern.rs

pub struct Pattern {
    pub id: String,

    pub description: String,

    pub occurrences: u32,

    pub confidence: f32,

    pub evidence: Vec<String>,
}
