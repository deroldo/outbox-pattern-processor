use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow, Clone, PartialEq)]
pub struct SqsDestination {
    pub queue_url: String,
}
