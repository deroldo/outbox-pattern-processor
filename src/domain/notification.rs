use crate::domain::outbox::Outbox;

#[derive(Clone, Default)]
pub struct NotificationResult {
    pub sent: Vec<Outbox>,
    pub failed: Vec<Outbox>,
}
