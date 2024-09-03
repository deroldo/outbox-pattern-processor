use std::fmt;

#[derive(Debug)]
pub struct OutboxPatternProcessorError {
    pub status_code: u16,
    pub cause: String,
    pub message: Option<String>,
}

impl OutboxPatternProcessorError {
    pub fn new(
        cause: &str,
        message: &str,
    ) -> Self {
        Self {
            status_code: 500,
            cause: cause.to_string(),
            message: Some(message.to_string()),
        }
    }
}

impl std::error::Error for OutboxPatternProcessorError {}

impl fmt::Display for OutboxPatternProcessorError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}
