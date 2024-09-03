use crate::app_state::AppState;
use crate::error::OutboxPatternProcessorError;
use crate::notification::NotificationResult;
use crate::outbox::Outbox;
use crate::outbox_group::GroupedOutboxed;
use aws_sdk_sns::error::ProvideErrorMetadata;
use aws_sdk_sns::types::{MessageAttributeValue, PublishBatchRequestEntry};
use tracing::log::error;

pub struct SnsNotificationService;

impl SnsNotificationService {
    pub async fn send(
        app_state: &AppState,
        outboxes: &GroupedOutboxed,
    ) -> Result<NotificationResult, OutboxPatternProcessorError> {
        let mut notification_result = NotificationResult::default();

        let sns_client = if let Some(client) = app_state.sns_client.clone() {
            client
        } else {
            notification_result.failed.extend(outboxes.sns.values().flat_map(|it| it.clone()).collect::<Vec<_>>());
            return Ok(notification_result);
        };

        for (topic_arn, topic_outboxes) in outboxes.sns.clone() {
            let chunks = topic_outboxes.chunks(10).collect::<Vec<&[Outbox]>>();

            for chunk in chunks {
                let mut entries = vec![];
                let mut outbox_entries = vec![];
                for outbox in chunk {
                    let idempotent_key_attribute_value_result = attribute_value(outbox, &outbox.idempotent_key.to_string());
                    if idempotent_key_attribute_value_result.is_err() {
                        notification_result.failed.push(outbox.clone());
                        let error = idempotent_key_attribute_value_result.expect_err("Failed to get expect idempotent_key_attribute_value error");
                        error!(
                            "{} - Cause: {}",
                            error.message.unwrap_or("Failed to create idempotent_key_attribute_value".to_string()),
                            error.cause
                        );
                        break;
                    }

                    let idempotent_key_attribute_value = idempotent_key_attribute_value_result.expect("Failed to get expect idempotent_key_attribute_value");

                    let mut entry_builder = PublishBatchRequestEntry::builder()
                        .id(outbox.idempotent_key)
                        .message(outbox.payload.clone())
                        .message_attributes("x-idempotent-key", idempotent_key_attribute_value);

                    if let Some(headers) = outbox.headers.clone() {
                        for (key, value) in headers.0 {
                            let attribute_value_result = attribute_value(outbox, &value);
                            if attribute_value_result.is_err() {
                                notification_result.failed.push(outbox.clone());
                                let error = attribute_value_result.expect_err("Failed to get expect attribute_value error");
                                error!("{} - Cause: {}", error.message.unwrap_or("Failed to create attribute_value".to_string()), error.cause);
                                break;
                            }

                            let attribute_value = attribute_value_result.expect("Failed to get expect attribute_value");
                            entry_builder = entry_builder.message_attributes(key, attribute_value);
                        }
                    }

                    let entry = entry_builder.build().map_err(|error| {
                        OutboxPatternProcessorError::new(
                            &error.to_string(),
                            &format!("Failed to create batch entry for outbox idempotent_key={}", outbox.idempotent_key),
                        )
                    })?;

                    outbox_entries.push(outbox.clone());
                    entries.push(entry);
                }

                let publish_result = sns_client
                    .client
                    .publish_batch()
                    .topic_arn(&topic_arn)
                    .set_publish_batch_request_entries(Some(entries))
                    .send()
                    .await
                    .map_err(|error| {
                        let body = error
                            .raw_response()
                            .map(|rr| rr.body())
                            .map(|body| {
                                if let Some(bytes) = body.bytes() {
                                    String::from_utf8(bytes.to_vec()).ok().unwrap_or(String::from("Unknown: Failed to convert bytes to string"))
                                } else {
                                    String::from("Unknown: None bytes")
                                }
                            })
                            .unwrap_or(String::from("Unknown"));

                        OutboxPatternProcessorError::new(&body, error.message().unwrap_or("Failed to publish sns batch"));
                    });

                if publish_result.is_ok() {
                    notification_result.sent.extend(outbox_entries);
                } else {
                    notification_result.failed.extend(outbox_entries);
                }
            }
        }

        Ok(notification_result)
    }
}

fn attribute_value(
    outbox: &Outbox,
    value: &str,
) -> Result<MessageAttributeValue, OutboxPatternProcessorError> {
    MessageAttributeValue::builder().data_type("String").string_value(value).build().map_err(|error| {
        OutboxPatternProcessorError::new(
            &error.to_string(),
            &format!(
                "Failed to create message attribute with value={} for outbox idempotent_key={}",
                value, outbox.idempotent_key
            ),
        )
    })
}
