use crate::models::appstate::AppstateWrapper;
use crate::models::user::AuthUser;
use crate::util::jwt::claims::Claims;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Body {
    old_password: String,
    new_password: String,
}

/// Changes Password to new one, dependent on old password confirmation
/// Generates new token for user
#[axum_macros::debug_handler]
pub async fn change_password(
    auth_user: Extension<AuthUser>,
    State(appstate): State<AppstateWrapper>,
    jar: PrivateCookieJar,
    Json(body): Json<Body>
) -> Result<(StatusCode, PrivateCookieJar), (StatusCode, &'static str)> {
    let mut user = auth_user.0.0;
    let appstate = appstate.0;

    // confirm old password
    match user.compare_passwords(body.old_password) {
        Ok(o) => {
            if !o {
                return Err((StatusCode::UNAUTHORIZED, "Wrong Password"))
            }
        },
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to compare passwords"))
    };

    // hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let new_hashed = match argon2.hash_password(body.new_password.as_ref(), &salt) {
        Ok(o) => o.to_string(),
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash new password"))
    };

    // generate new token-id
    let new_tokenid = Uuid::new_v4();
    user.tokenid = new_tokenid;

    // update new password in db
    let conn = &appstate.db_pool;
    let query = r"UPDATE users SET password = $1, tokenid = $2 WHERE uuid = $3";
    let query_result = sqlx::query(query)
        .bind(new_hashed)
        .bind(new_tokenid.to_string())
        .bind(user.uuid.to_string())
        .execute(conn.as_ref())
        .await;

    if !query_result.is_ok() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write change to db"))
    }

    // generate new token
    let token = match Claims::generate_jwt(&appstate.jwt_secret, &user) {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate new token"))
    };

    // add new token to cookies
    let mut cookie = Cookie::new("token", token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);

    let jar = jar.add(cookie);

    Ok((StatusCode::OK, jar))
}