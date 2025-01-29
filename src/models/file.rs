use std::error::Error;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::appstate::Appstate;

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub reference_uuid: Uuid,
    pub owner_uuid: Uuid,
    pub filename: String,

    pub timestamp: u64,
}


impl File {
    pub fn new(owner_uuid: Uuid, filename: String) -> Self {
        Self {
            reference_uuid: Uuid::new_v4(),
            owner_uuid,
            filename,
            timestamp: Utc::now().timestamp() as u64,
        }
    }

    pub async fn write_to_db(&self, appstate: &Appstate) -> Result<(), Box<dyn Error>> {
        let conn = &appstate.db_pool;

        let query = r"INSERT INTO file (reference_uuid, owner_uuid, filename) VALUES ($1, $2, $3)";
        let query = sqlx::query(query)
            .bind(&self.reference_uuid.to_string())
            .bind(&self.owner_uuid.to_string())
            .bind(&self.filename)
            .execute(conn)
            .await?;


        Ok(())
    }
}