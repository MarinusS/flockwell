#[derive(Debug, PartialEq, Eq)]
pub struct Animal {
    pub id: String,
    pub tag: Option<String>,
}

impl Animal {
    pub fn new(id: &str, tag: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            tag: tag.map(str::to_string),
        }
    }
}
