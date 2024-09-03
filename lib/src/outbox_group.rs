use std::collections::HashMap;
use crate::outbox::Outbox;

#[derive(Clone, Default)]
pub struct GroupedOutboxed {
    pub sqs: HashMap<String, Vec<Outbox>>,
    pub sns: HashMap<String, Vec<Outbox>>,
    pub http: Vec<Outbox>,
}