use crate::models::appstate::AppstateWrapper;
use crate::models::user::AuthUser;
use crate::util::jwt::claims::Claims;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;

#[axum_macros::debug_handler]
pub async fn refresh_token(
    auth_user: Extension<AuthUser>,
    jar: PrivateCookieJar,
    State(appstate): State<AppstateWrapper>,
) -> Result<(StatusCode, PrivateCookieJar), (StatusCode, String)> {
    let appstate = appstate.0;
    let user = auth_user.0.0;

    // generate new token
    let new_token = match Claims::generate_jwt(&appstate.jwt_secret, &user) {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate new token".to_string()))
    };

    // set new token in cookies
    let mut cookie = Cookie::new("token", new_token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);

    let jar = jar.add(cookie);

    Ok((StatusCode::OK, jar))
}