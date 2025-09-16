#[derive(Debug, Clone, PartialEq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    None,
}

impl BumpType {
    pub fn from_conventional_commit(message: &str) -> Self {
        let message = message.to_lowercase();

        if message.starts_with("feat!") || message.contains("!") {
            BumpType::Major
        } else if message.starts_with("feat") {
            BumpType::Minor
        } else if message.starts_with("fix") || message.starts_with("perf") {
            BumpType::Patch
        } else {
            BumpType::None
        }
    }
}
