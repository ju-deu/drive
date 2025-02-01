use crate::models::appstate::Appstate;
use crate::models::user::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use std::error::Error;
use std::io;
use std::path::Path;
use tokio_stream::Stream;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub reference_uuid: Uuid,
    pub owner_uuid: Uuid,
    pub filename: String,

    pub relative_path: String, /* Can't use Path as it doesn't implement Clone */
    pub absolute_path: String, /* Can't use Path as it doesn't implement Clone */
    /// Filesize in bytes
    pub size: usize,

    pub timestamp: usize,
}
// todo: create and enter?
//

impl File {
    /// returns File model without validation
    pub fn new( reference_uuid: Uuid, owner_uuid: Uuid, filename: String, relative_path: String, absolute_path: String, size: usize)
    -> Self {
        Self {
            reference_uuid,
            owner_uuid,
            filename,
            relative_path,
            absolute_path,
            size,
            timestamp: Utc::now().timestamp() as usize,
        }
    }
    /// if valid files returns Some(file) else None
    pub async fn construct(
        reference_uuid: Option<Uuid>,
        filename: String,
        user: &User,
        size: usize,
        appstate: &Appstate,
    ) -> Option<Self> {
        let ref_id = reference_uuid.unwrap_or(Uuid::new_v4());
        let relative_path = format!("{}/{}", user.uuid, ref_id);
        let absolute_path = format!("{}/{}", &appstate.file_location, relative_path);
        let file = Self {
            reference_uuid: ref_id,
            owner_uuid: user.uuid,
            filename,
            relative_path,
            absolute_path,
            size,
            timestamp: Utc::now().timestamp() as usize,
        };
        // make sure its valid
        match file.is_valid(appstate).await {
            Ok(o) => if o { Some(file) } else { None },
            _ => None,
        }
    }

    /// just some bare-bones validation
    pub async fn is_valid(&self, appstate: &Appstate) -> Result<bool, Box<dyn Error>> {
        /*if self.filename.is_empty() || self.filename.contains("/") || self.filename.contains(r"\") {
            return Ok(false)
        }*/
        // check that absolute path and relative path are correct
        if self.absolute_path != format!("{}/{}", appstate.file_location,self.relative_path) {
            return Ok(false)
        }
        // check for existence or create it
        let path = Path::new(&self.absolute_path);
        let parent = match path.parent() {
            Some(o) => o,
            _ => return Ok(false),
        };
        if !tokio::fs::try_exists(parent).await? {
            // create dir as it doesn't exist
            tokio::fs::create_dir(parent).await?;
        }

        // check for file size under 100MB
        if self.size > 100000000_usize { /* 100M bytes = 100 MB */
            return Ok(false)
        }
        Ok(true)
    }

    /// Maps PgRow to File \
    /// DOES NOT CHECK FOR VALIDATION
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


    /// writes self to db connection from appstate \
    /// DOES NOT CHECK FOR VALIDATION
    pub async fn write_to_db(&self, appstate: &Appstate) -> Result<(), Box<dyn Error>> {
        let conn = &appstate.db_pool;

        let query = r"INSERT INTO file (reference_uuid, owner_uuid, filename, relative_path, absolute_path, size)
                         VALUES ($1, $2, $3, $4, $5, $6)";
        let _query = sqlx::query(query)
            .bind(&self.reference_uuid.to_string())
            .bind(&self.owner_uuid.to_string())
            .bind(&self.filename)
            .bind(&self.relative_path)
            .bind(&self.absolute_path)
            .bind(self.size.clone() as i64)
            .execute(conn.as_ref())
            .await?;

        Ok(())
    }
    /// retrieves self from db by reference uuid \
    /// DOES NOT CHECK FOR VALIDATION
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

    /// deletes file row from db by reference_uuid, owner_uuid, and filename
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
    /// NOT RECOMMENDED FOR LARGE FILES! use stream instead\
    /// DOES NOT CHECK FOR VALIDATION \
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

    /// stream files to disk (recommended for large files) \
    /// DOES NOT CHECK FOR VALIDATION
    pub async fn stream_disk<S>(&self, mut stream: S) -> io::Result<()>
    where
        S: Stream<Item = io::Result<Vec<u8>>> + Unpin,
    {
        /*let path = Path::new(&self.absolute_path);
        // create file
        let file = fs::File::create(path);

        let mut writer = AsyncW::new(stream);
        while let Some(chunk) = stream.next().await {
            let data = chunk?;
            writer
        }*/
        panic!("TODO: method `stream_disk<S>` on `File`");

        Ok(())
    }
}
