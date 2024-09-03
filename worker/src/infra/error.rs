use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub status_code: u16,
    pub cause: String,
    pub message: Option<String>,
}

impl AppError {
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

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}
