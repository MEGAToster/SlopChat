#[derive(Debug, Clone)]
pub struct Hit {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published: Option<String>,
}

#[derive(Debug, Default)]
pub struct Batch {
    pub hits: Vec<Hit>,
    pub units: u32,
    pub cost_usd: f64,
}
