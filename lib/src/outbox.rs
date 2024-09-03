use crate::http_destination::HttpDestination;
use crate::outbox_destination::OutboxDestination;
use crate::sns_destination::SnsDestination;
use crate::sqs_destination::SqsDestination;
use serde_json::Value;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, FromRow, Clone, PartialEq)]
pub struct Outbox {
    pub idempotent_key: Uuid,
    pub partition_key: Uuid,
    pub destinations: Json<Vec<OutboxDestination>>,
    pub headers: Option<Json<HashMap<String, String>>>,
    pub payload: String,
    pub created_at: DateTime<Utc>,
    pub processing_until: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl Outbox {
    pub fn http_post_json(
        partition_key: Uuid,
        url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &Value,
    ) -> Self {
        Self::http(partition_key, url, headers, &payload.to_string(), None)
    }

    pub fn http_put_json(
        partition_key: Uuid,
        url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &Value,
    ) -> Self {
        Self::http(partition_key, url, headers, &payload.to_string(), Some("put".to_string()))
    }

    pub fn http_patch_json(
        partition_key: Uuid,
        url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &Value,
    ) -> Self {
        Self::http(partition_key, url, headers, &payload.to_string(), Some("patch".to_string()))
    }

    pub fn sqs(
        partition_key: Uuid,
        queue_url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &str,
    ) -> Self {
        let destinations = vec![OutboxDestination::SqsDestination(SqsDestination { queue_url: queue_url.to_string() })];

        Self::new(partition_key, destinations, headers, payload)
    }

    pub fn sns(
        partition_key: Uuid,
        topic_arn: &str,
        headers: Option<HashMap<String, String>>,
        payload: &str,
    ) -> Self {
        let destinations = vec![OutboxDestination::SnsDestination(SnsDestination { topic_arn: topic_arn.to_string() })];

        Self::new(partition_key, destinations, headers, payload)
    }

    fn http(
        partition_key: Uuid,
        url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &str,
        method: Option<String>,
    ) -> Self {
        let mut extended_headers = headers.unwrap_or_default();
        extended_headers.extend(HashMap::from([("Content-Type".to_string(), "application/json".to_string())]));

        Self::new(
            partition_key,
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: url.to_string(),
                headers: Some(extended_headers),
                method,
            })],
            None,
            payload,
        )
    }

    fn new(
        partition_key: Uuid,
        destinations: Vec<OutboxDestination>,
        headers: Option<HashMap<String, String>>,
        payload: &str,
    ) -> Self {
        Outbox {
            idempotent_key: Uuid::now_v7(),
            partition_key,
            destinations: Json(destinations),
            headers: headers.map(Json),
            payload: payload.to_string(),
            created_at: Utc::now(),
            processing_until: None,
            processed_at: None,
        }
    }
}
