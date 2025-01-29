use axum::Extension;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::appstate::{Appstate, AppstateWrapper};
use crate::models::user::AuthUser;

#[derive(Serialize, Deserialize)]
struct Response {
    reference_uuid: Uuid,

}



#[axum_macros::debug_handler]
pub async fn upload(
    auth_user: Extension<AuthUser>,
    State(appstate): State<AppstateWrapper>,
    mut multipart: Multipart
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let appstate = appstate.0;
    let user = auth_user.0.0;

    while let Some(mut field) = multipart.next_field().await.ok()
        .ok_or((StatusCode::BAD_REQUEST, "Failed to get next field"))? {
        // get name and content
        let name = field.name()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get field name"))?.to_string();
        let data = field.bytes().await
            .ok().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get data"))?;


        // continue on everything that isn't marked as a file
        if name != "file".to_string() {
            continue;
        }

        // TODO! WRITE FILE

        println!("Length of `{}` is {} bytes", name, data.len());
    }


    Ok(StatusCode::CREATED)
}