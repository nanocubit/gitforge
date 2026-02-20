#[derive(Default)]
pub struct BpgtAgent {
    db_path: String,
}
impl BpgtAgent {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }
    pub async fn process_voice(&self, text: &str) -> Result<String, String> {
        Ok(format!(
            "BPGT agent accepted voice input for '{}': {}",
            self.db_path, text
        ))
    }
}
