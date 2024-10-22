use crate::app_state::AppState;
use crate::environment::Environment;
use crate::error::OutboxPatternProcessorError;
use crate::notification::NotificationResult;
use crate::outbox_destination::OutboxDestination;
use crate::outbox_group::GroupedOutboxed;
use regex::Regex;
use tracing::instrument;
use tracing::log::error;

pub struct HttpNotificationService;

impl HttpNotificationService {
    #[instrument(skip_all, name = "send_to_http")]
    pub async fn send(
        app_state: &AppState,
        outboxes: &GroupedOutboxed,
    ) -> Result<NotificationResult, OutboxPatternProcessorError> {
        let mut notification_result = NotificationResult::default();

        for outbox in outboxes.http.clone() {
            for destination in outbox.destinations.0.clone() {
                if let OutboxDestination::HttpDestination(http) = destination {
                    let method = http.method.unwrap_or("POST".to_string()).to_uppercase();
                    let mut request = match method.as_str() {
                        "PUT" => app_state.http_gateway.client.put(&http.url),
                        "PATCH" => app_state.http_gateway.client.patch(&http.url),
                        _ => app_state.http_gateway.client.post(&http.url),
                    };

                    if let Some(headers) = http.headers {
                        for (key, value) in headers {
                            if let Ok(regex) = Regex::new("^\\{\\{[A-Z_]+}}$") {
                                if regex.is_match(&value) {
                                    let normalized_value_env_name = value.replace(['{', '}'], "");
                                    let env_value = Environment::string(&normalized_value_env_name, &value);
                                    request = request.header(key, env_value);
                                } else {
                                    request = request.header(key, value);
                                }
                            } else {
                                request = request.header(key, value);
                            }
                        }
                    }

                    if let Some(headers) = outbox.headers.clone() {
                        for (key, value) in headers.0 {
                            request = request.header(key, value);
                        }
                    }

                    request = request.header("x-idempotent-key", outbox.idempotent_key.to_string());

                    let result = request.body(outbox.payload.clone()).send().await;

                    if let Ok(response) = result {
                        if response.status().is_success() {
                            notification_result.sent.push(outbox.clone());
                        } else {
                            notification_result.failed.push(outbox.clone());
                            error!(
                                "Failed to send http notification for idempotent_key {} with status {} and body {}",
                                outbox.idempotent_key,
                                response.status(),
                                response.text().await.unwrap_or("unknown".to_string())
                            );
                        }
                    } else {
                        notification_result.failed.push(outbox.clone());
                        error!(
                            "Failed to send http notification cause {}",
                            result.err().map(|error| error.to_string()).unwrap_or("unknown".to_string())
                        );
                    }
                }
            }
        }

        Ok(notification_result)
    }
}
