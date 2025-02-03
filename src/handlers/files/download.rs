use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use std::path::Path as StdPath;
use tower_http::services::ServeDir;
use uuid::Uuid;

pub async fn serve_file(
    State(appstate): State<AppstateWrapper>,
    auth_user: Extension<AuthUser>,
    Path(ref_id): Path<Uuid>,
) -> Result<Response, (StatusCode, &'static str)> {
    let user = auth_user.0.0;
    let appstate = appstate.0;

    // check that user owns file
    let file = match File::get_from_db(ref_id, user.uuid.clone(), &appstate).await {
        Ok(row) => row,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Failed to find in db"))
    };

    // check again that the file exists
    let path = StdPath::new(&file.absolute_path);
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "Failed to find file on disk"))
    }

    // construct real filename to not serve filed named after uuids
    let actual_filename = format!("{}/{}/{}", &appstate.file_location, &user.uuid.to_string(), file.filename);

    // serve file with respective filename
    let service = ServeDir::new(file.absolute_path);
    match service.try_call(actual_filename).await {
        Ok(o) => Ok(o.into_response()),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file"))
    }
}