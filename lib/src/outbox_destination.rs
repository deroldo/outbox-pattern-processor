use crate::http_destination::HttpDestination;
use crate::sns_destination::SnsDestination;
use crate::sqs_destination::SqsDestination;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OutboxDestination {
    SqsDestination(SqsDestination),
    SnsDestination(SnsDestination),
    HttpDestination(HttpDestination),
}
