// /src/experience/reflection/review.rs

pub struct ReflectionReview {
    pub id: String,

    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,

    pub reflections: Vec<String>,

    pub summary: String,
}
