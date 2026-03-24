use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ErrorOutput {
    pub code: &'static str,
    pub message: String,
}
