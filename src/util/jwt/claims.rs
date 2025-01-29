use std::error::Error;
use std::sync::Arc;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::user::User;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Claims {
    pub(crate) sub: Uuid,
    pub(crate) tokenid: Uuid,
    pub(crate) iat: usize,
    pub(crate) exp: usize,
}


impl Claims {
    /// Validates Claims and returns User if valid
    pub async fn validate_claims(
        &self, conn: &Arc<Pool<Postgres>>
    ) -> Result<Option<User>, Box<dyn Error>> {

        // check for timestamps
        if self.exp < Utc::now().timestamp() as usize {
            return Ok(None)
        }

        // get user from db
        // use query_as macro instead (can't figure it out)
        let query = r"SELECT * FROM users WHERE uuid = $1";
        let row = sqlx::query(query)
            .bind(&self.sub.to_string())
            .fetch_one(conn.as_ref())
            .await?;

        let user = User::from_pg_row(row)?;

        // compare ids
        if user.tokenid != self.tokenid {
            return Ok(None)
        }

        Ok(Some(user))
    }

    pub fn generate_jwt(jwt_secret: &String, user: &User) -> jsonwebtoken::errors::Result<String> {
        let claims = Claims {
            sub: user.uuid,
            tokenid: user.tokenid,
            iat: Utc::now().timestamp() as usize,
            exp: Utc::now().timestamp() as usize + 31536000, /* 1 year */
        };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))
    }
}