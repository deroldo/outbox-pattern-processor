use crate::outbox::Outbox;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct GroupedOutboxed {
    pub sqs: HashMap<String, Vec<Outbox>>,
    pub sns: HashMap<String, Vec<Outbox>>,
    pub http: Vec<Outbox>,
}
