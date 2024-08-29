use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, FromRow, Clone, PartialEq)]
pub struct HttpDestination {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub method: Option<String>,
}
