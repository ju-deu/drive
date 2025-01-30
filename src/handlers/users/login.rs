use crate::models::appstate::AppstateWrapper;
use crate::models::user::User;
use crate::util::jwt::claims::Claims;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(appstate): State<AppstateWrapper>,
    jar: PrivateCookieJar,
    Json(body): Json<Body>
) -> Result<(StatusCode, PrivateCookieJar), (StatusCode, &'static str)> {
    let appstate = appstate.0;

    // get user from db
    let conn = &appstate.db_pool;
    let query_result = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(body.username)
        .fetch_optional(conn.as_ref())
        .await;

    let row = match query_result {
        Ok(Some(row)) => row,
        Ok(None) => return Err((StatusCode::BAD_REQUEST, "User does not exist")),
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user from db"))
    };

    // compare passwords
    let user= User::from_pg_row(row)
        .ok().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse user"))?;

    match user.compare_passwords(body.password) {
        Ok(o) => {
            if !o {
                return Err((StatusCode::UNAUTHORIZED, "Wrong password"))
            }
        },
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to compare passwords"))
    }

    // generate token
    let token = match Claims::generate_jwt(&appstate.jwt_secret, &user) {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate jwt"))
    };

    // set cookies
    let mut cookie = Cookie::new("token", token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);

    let jar = jar.add(cookie);

    Ok((StatusCode::OK, jar))
}