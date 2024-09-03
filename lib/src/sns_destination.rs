use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow, Clone, PartialEq)]
pub struct SnsDestination {
    pub topic_arn: String,
}
