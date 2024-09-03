use crate::outbox_destination::OutboxDestination;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::FromRow;
use std::collections::HashMap;
use serde_json::Value;
use uuid::Uuid;
use crate::http_destination::HttpDestination;
use crate::sns_destination::SnsDestination;
use crate::sqs_destination::SqsDestination;

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
    pub fn http(
        partition_key: Uuid,
        url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &str,
    ) -> Self {
        Outbox {
            idempotent_key: Uuid::now_v7(),
            partition_key,
            destinations: Json(vec![OutboxDestination::HttpDestination(HttpDestination {
                url: url.to_string(),
                headers: None,
                method: None,
            })]),
            headers: headers.map(Json),
            payload: payload.to_string(),
            created_at: Utc::now(),
            processing_until: None,
            processed_at: None,
        }
    }

    pub fn http_json(
        partition_key: Uuid,
        url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &Value,
    ) -> Self {
        Outbox {
            idempotent_key: Uuid::now_v7(),
            partition_key,
            destinations: Json(vec![OutboxDestination::HttpDestination(HttpDestination {
                url: url.to_string(),
                headers: Some(HashMap::from([("Content-Type".to_string(), "application/json".to_string())])),
                method: None,
            })]),
            headers: headers.map(Json),
            payload: payload.to_string(),
            created_at: Utc::now(),
            processing_until: None,
            processed_at: None,
        }
    }

    pub fn sqs(
        partition_key: Uuid,
        queue_url: &str,
        headers: Option<HashMap<String, String>>,
        payload: &Value,
    ) -> Self {
        Outbox {
            idempotent_key: Uuid::now_v7(),
            partition_key,
            destinations: Json(vec![OutboxDestination::SqsDestination(SqsDestination {
                queue_url: queue_url.to_string(),
            })]),
            headers: headers.map(Json),
            payload: payload.to_string(),
            created_at: Utc::now(),
            processing_until: None,
            processed_at: None,
        }
    }

    pub fn sns(
        partition_key: Uuid,
        topic_arn: &str,
        headers: Option<HashMap<String, String>>,
        payload: &Value,
    ) -> Self {
        Outbox {
            idempotent_key: Uuid::now_v7(),
            partition_key,
            destinations: Json(vec![OutboxDestination::SnsDestination(SnsDestination {
                topic_arn: topic_arn.to_string(),
            })]),
            headers: headers.map(Json),
            payload: payload.to_string(),
            created_at: Utc::now(),
            processing_until: None,
            processed_at: None,
        }
    }
}
