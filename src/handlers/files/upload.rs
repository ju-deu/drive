use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Response {
    reference_uuid: Uuid,
    file: String
}

#[axum_macros::debug_handler]
pub async fn upload(
    auth_user: Extension<AuthUser>,
    State(appstate): State<AppstateWrapper>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<Response>>), (StatusCode, &'static str)> {
    let appstate = appstate.0;
    let user = auth_user.0 .0;

    let mut response_references = Vec::new();

    while let Ok(Some(field)) = multipart
        .next_field()
        .await
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

        // get file data
        let filename = &field
            .file_name()
            .ok_or((
                StatusCode::BAD_REQUEST,
                "Failed to get filename (most likely due to no file being sent or embedded file content is being used)",
            ))?
            .to_string();

        // get file content
        let content = field
            .bytes()
            .await
            .ok()
            .ok_or((StatusCode::BAD_REQUEST, "Failed to get file content (most likely because file is too large)"))?;

        // construct file and paths
        let file = File::construct(
            None,
            filename.clone(),
            &user,
            content.len(),
            &appstate,
        ).await.ok_or((StatusCode::BAD_REQUEST, "Failed to construct File model"))?;

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
                        file.reference_uuid,
                        file.owner_uuid,
                    );
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to disk and remove from db"));
                }
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file to disk"));
            }
        }

        response_references.push(Response { reference_uuid: file.reference_uuid, file: filename.to_owned() })
    }

    Ok((StatusCode::CREATED, Json(response_references)))
}
