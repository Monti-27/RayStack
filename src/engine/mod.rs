pub mod listener;
pub mod handler;

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub signature: String,
    pub logs: Vec<String>,
}
