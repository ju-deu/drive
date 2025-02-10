use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Extension;
use uuid::Uuid;

pub async fn delete_file(
    State(appstate): State<AppstateWrapper>,
    auth_user: Extension<AuthUser>,
    Path(ref_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let appstate = appstate.0;
    let user = auth_user.0.0;
    // get file data from db
    let file = match File::get_from_db(ref_id.clone(), user.uuid.clone(), &appstate).await {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Failed to find file in db")),
    };

    // delete file from disk and db
    match file.delete_from_disk().await {
        Ok(_) => {},
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete from disk")),
    }

    match file.delete_from_db(&appstate).await {
        Ok(_) => {},
        Err(_) => {
            eprintln!("FATAL: DANGLING ENTRY IN `file`, file: {:?}", file);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete from db"))
        }
    }


    Ok(StatusCode::NO_CONTENT)
}