use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}