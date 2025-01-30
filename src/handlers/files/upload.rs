use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Response {
    reference_uuid: Uuid,
}

#[axum_macros::debug_handler]
pub async fn upload(
    auth_user: Extension<AuthUser>,
    State(appstate): State<AppstateWrapper>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Response>), (StatusCode, &'static str)> {
    let appstate = appstate.0;
    let user = auth_user.0 .0;

    let reference_uuid = Uuid::new_v4();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .ok()
        .ok_or((StatusCode::BAD_REQUEST, "Failed to get next field"))?
    {
        // get name, content and filename
        let field_name = &field
            .name()
            .ok_or((StatusCode::BAD_REQUEST, "Failed to get field name"))?
            .to_string();

        // continue on everything that isn't marked as a file
        if field_name.to_lowercase() != "file" {
            continue;
        }

        let filename = &field
            .file_name()
            .ok_or((
                StatusCode::BAD_REQUEST,
                "Failed to get filename (most likely due to no file being sent)",
            ))?
            .to_string();
        if filename.contains("/") { return Err((StatusCode::BAD_REQUEST, "Filename includes \"/\""))}

        let file_extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        // TODO ! use streaming instead
        let content = field
            .bytes()
            .await
            .ok()
            .ok_or((StatusCode::BAD_REQUEST, "Failed to get file content"))?;

        // construct file and paths
        let string_relative_file_path = format!(
            "{}/{}.{}",
            &user.username,
            &reference_uuid.to_string(),
            &file_extension
        );
        let string_absolute_file_path = format!(
            "{}/{}",
            &appstate.file_location,
            &string_relative_file_path
        );

        let file = File::new(
            Some(reference_uuid.clone()),
            user.uuid.clone(),
            filename.clone(),
            string_relative_file_path,
            string_absolute_file_path,
            content.len(),
        );

        // write to db
        let _query = file.write_to_db(&appstate)
            .await.ok().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to db"));

        // write to disk
        match file.write_disk(content).await {
            Ok(_) => {}
            Err(_) => {
                // Delete from DB on disk write failure
                if let Err(_) = file.delete_from_db(&appstate).await {
                    eprintln!(
                        "FATAL! Dangling entry in database 'file': reference_uuid: {} | owner_uuid: {}",
                        reference_uuid,
                        file.owner_uuid,
                    );
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to disk and remove from db"));
                }
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file to disk"));
            }
        }
    }

    Ok((StatusCode::CREATED, Json(Response{ reference_uuid, })))
}
