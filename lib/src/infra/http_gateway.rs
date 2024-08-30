use std::time::Duration;

use crate::infra::error::AppError;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

#[derive(Clone)]
pub struct HttpGateway {
    pub client: ClientWithMiddleware,
}

impl HttpGateway {
    pub fn new(request_timeout: u64) -> Result<Self, AppError> {
        let client = ClientBuilder::new(
            Client::builder()
                .timeout(Duration::from_secs(request_timeout))
                .build()
                .map_err(|error| AppError::new(&error.to_string(), "Failed to create http gateway client"))?,
        )
        .build();

        Ok(Self { client })
    }
}
