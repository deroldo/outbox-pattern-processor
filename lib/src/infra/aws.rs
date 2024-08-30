use std::borrow::Cow;
use std::env;

use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_config::Region;

#[derive(Clone)]
pub struct SnsClient {
    pub client: aws_sdk_sns::Client,
}

#[derive(Clone)]
pub struct SqsClient {
    pub client: aws_sdk_sqs::Client,
}

impl SnsClient {
    pub async fn new(aws_config: &aws_config::SdkConfig) -> SnsClient {
        let endpoint = env::var("LOCAL_ENDPOINT").ok();
        let region = env::var("LOCAL_REGION").map(|region| Region::new(Cow::Owned(region))).ok();

        let client = match endpoint {
            None => aws_sdk_sns::Client::new(aws_config),
            Some(url) => aws_sdk_sns::Client::from_conf(
                aws_sdk_sns::config::Builder::from(aws_config)
                    .endpoint_url(url)
                    .region(region)
                    .credentials_provider(DefaultCredentialsChain::builder().build().await)
                    .build(),
            ),
        };

        SnsClient { client }
    }
}

impl SqsClient {
    pub async fn new(aws_config: &aws_config::SdkConfig) -> SqsClient {
        let endpoint = env::var("LOCAL_ENDPOINT").ok();
        let region = env::var("LOCAL_REGION").map(|region| Region::new(Cow::Owned(region))).ok();

        let client = match endpoint {
            None => aws_sdk_sqs::Client::new(aws_config),
            Some(url) => aws_sdk_sqs::Client::from_conf(
                aws_sdk_sqs::config::Builder::from(aws_config)
                    .endpoint_url(url)
                    .region(region)
                    .credentials_provider(DefaultCredentialsChain::builder().build().await)
                    .build(),
            ),
        };

        SqsClient { client }
    }
}
