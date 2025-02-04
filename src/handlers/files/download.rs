use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;
use axum::extract::{Path, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::Extension;
use std::path::Path as StdPath;
use axum::response::IntoResponse;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use uuid::Uuid;

#[axum_macros::debug_handler]
pub async fn serve_file(
    State(appstate): State<AppstateWrapper>,
    auth_user: Extension<AuthUser>,
    Path(ref_id): Path<Uuid>,
    req: Request,
) -> Result<axum_core::response::Response, (StatusCode, &'static str)> {
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

    // construct ServeFile and response
    let service = ServeFile::new(&file.absolute_path);

    let mut response = service.oneshot(req).await.map_err(|_| (StatusCode::NOT_FOUND, "Failed to construct file service"))?;

    // set custom headers for original filename
    let header_value = format!("attachment; filename=\"{}\"", &file.filename);
    if let Ok(value) = HeaderValue::from_str(&header_value) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, value);
    } else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to construct headers"))
    }

    Ok(response.into_response())
}