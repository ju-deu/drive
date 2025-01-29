use std::error::Error;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
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
    /// Maps PgRow to File
    pub fn from_pg_row(row: PgRow) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            reference_uuid: Uuid::parse_str(row.try_get("reference_uuid")?)?,
            owner_uuid: Uuid::parse_str(row.try_get("owner_uuid")?)?,
            filename: row.try_get("filename")?,
            timestamp: row.try_get("timestamp") as u64,
        })
    }
    /// writes self to db connection from appstate
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
    /// retrieves self from db by reference uuid
    pub async fn get_from_db(reference_uuid: Uuid, appstate: &Appstate) -> Result<Self, Box<dyn Error>> {
        let conn = &appstate.db_pool;

        let query = r"SELECT * FROM file WHERE reference_uuid = $1";
        let row = sqlx::query(query)
            .bind(reference_uuid.to_string())
            .fetch_one(conn)
            .await?;

        let file = File::from_pg_row(row)?;
        Ok(file)
    }
}