use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

super::sql_enum! {
    pub enum DeployStatus {
        Pending,
        Running,
        Failed,
        Success,
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone, Default)]
pub struct Deploy {
    pub id: i64,
    pub repo: String,
    pub actor: String,
    pub sender: Option<String>,
    pub begin: i64,
    pub finish: i64,
    pub status: DeployStatus,
}
