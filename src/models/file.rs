use crate::models::appstate::Appstate;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use std::error::Error;
use std::path::Path;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub reference_uuid: Uuid,
    pub owner_uuid: Uuid,
    pub filename: String,

    pub relative_path: String,
    pub absolute_path: String,
    /// Filesize in bytes
    pub size: usize,

    pub timestamp: usize,
}

impl File {
    pub fn new(
        reference_uuid: Option<Uuid>,
        owner_uuid: Uuid,
        filename: String,
        relative_path: String,
        absolute_path: String,
        size: usize,
    ) -> Self {
        Self {
            reference_uuid: reference_uuid.unwrap_or(Uuid::new_v4()),
            owner_uuid,
            filename,
            relative_path,
            absolute_path,
            size,
            timestamp: Utc::now().timestamp() as usize,
        }
    }
    /// Maps PgRow to File
    pub fn from_pg_row(row: PgRow) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            reference_uuid: Uuid::parse_str(row.try_get("reference_uuid")?)?,
            owner_uuid: Uuid::parse_str(row.try_get("owner_uuid")?)?,
            filename: row.try_get("filename")?,
            relative_path: row.try_get("relative_path")?,
            absolute_path: row.try_get("absolute_path")?,
            size: row.try_get::<i64, _>("size")? as usize,
            timestamp: row.try_get::<i64, _>("timestamp")? as usize,
        })
    }
    /// writes self to db connection from appstate
    pub async fn write_to_db(&self, appstate: &Appstate) -> Result<(), Box<dyn Error>> {
        let conn = &appstate.db_pool;

        let query = r"INSERT INTO file (reference_uuid, owner_uuid, filename) VALUES ($1, $2, $3)";
        let _query = sqlx::query(query)
            .bind(&self.reference_uuid.to_string())
            .bind(&self.owner_uuid.to_string())
            .bind(&self.filename)
            .execute(conn.as_ref())
            .await?;

        Ok(())
    }
    /// retrieves self from db by reference uuid
    pub async fn get_from_db(
        reference_uuid: Uuid,
        appstate: &Appstate,
    ) -> Result<Self, Box<dyn Error>> {
        let conn = &appstate.db_pool;

        let query = r"SELECT * FROM file WHERE reference_uuid = $1";
        let row = sqlx::query(query)
            .bind(reference_uuid.to_string())
            .fetch_one(conn.as_ref())
            .await?;

        let file = File::from_pg_row(row)?;
        Ok(file)
    }

    pub async fn delete_from_db(&self, appstate: &Appstate) -> Result<(), Box<dyn Error>> {
        let conn = &appstate.db_pool;

        let query =
            r"DELETE FROM file WHERE reference_uuid = $1 AND owner_uuid = $2 AND filename = $3";
        let _query = sqlx::query(query)
            .bind(&self.reference_uuid.to_string())
            .bind(&self.owner_uuid.to_string())
            .bind(&self.filename)
            .execute(conn.as_ref())
            .await?;

        Ok(())
    }
    /// writes file data to path specified in params
    /// also creates dir if it doesn't exist already
    pub async fn write_disk(&self, content: impl AsRef<[u8]>)
        -> Result<(), Box<dyn Error + Send + Sync>> {
        let path = Path::new(&self.absolute_path);
        let parent_path = path.parent()
            .and_then(|p| p.to_str())
            .unwrap_or_default();

        // create dir if it doesn't exist
        if !tokio::fs::try_exists(parent_path).await? {
            tokio::fs::create_dir_all(parent_path).await?
        }

        let _ = tokio::fs::write(path, content).await?;
        Ok(())
    }
}
