use crate::domain::destination::http_destination::HttpDestination;
use crate::domain::destination::sns_destination::SnsDestination;
use crate::domain::destination::sqs_destination::SqsDestination;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OutboxDestination {
    SqsDestination(SqsDestination),
    SnsDestination(SnsDestination),
    HttpDestination(HttpDestination),
}
