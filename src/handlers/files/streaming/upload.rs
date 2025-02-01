use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Response {
    reference_uuid: Uuid,
    filename: String
}

pub async fn stream_upload(
    State(appstate): State<AppstateWrapper>,
    auth_user: Extension<AuthUser>,
    mut multipart: Multipart
) -> Result<(StatusCode, Json<Vec<Response>>), (StatusCode, &'static str)> {
    let appstate = appstate.0;
    let user = auth_user.0.0;

    let response: Vec<Response> = Vec::new();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = match &field.name() {
            Some(x) => x.to_string(),
            _ => { return Err((StatusCode::BAD_REQUEST, "Failed to get field name"))}
        };
        // continue on everything not marked as a file
        if &field_name.to_lowercase() != "file" { continue; }

        let filename = match &field.file_name() {
            Some(x) => x.to_string(),
            _ => { return Err((StatusCode::BAD_REQUEST, "Failed to get filename"))}
        };

        let file = File::construct(
            None,
            filename.clone(),
            &user,
            0,
            &appstate
        ).await.ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to construct File"))?;

        let _query = match file.write_to_db(&appstate).await {
            Ok(_) => {},
            Err(e) => {
                eprintln!("{}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to db"))
            }
        };

        // write to file in chunks
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Failed to get next chunk"))?
        {
            println!("received {} bytes", chunk.len());
            // init file
            let mut file_options = tokio::fs::File::options().append(true)
                .open(&file.absolute_path)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file"))?;
            // write to file
            match file_options.write(&chunk.as_ref()).await {
                Ok(_) => {}
                Err(e) => {
                    // remove file, remove from db, return err from handler
                    match tokio::fs::remove_file(Path::new(&file.absolute_path)).await {
                        Ok(_) => {
                            match file.delete_from_db(&appstate).await {
                                Ok(_) => {},
                                Err(err) => {
                                    eprintln!(
                                        "FATAL: DANGLING ENTRY IN DB `file`, ref_id: {}, owner_id: {}, err: {}",
                                        &file.reference_uuid, file.owner_uuid, err
                                    );
                                },

                            }

                            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to file -> deleted"))
                        },
                        Err(e) => {
                            eprintln!("FATAL: FAILED TO DELETE FILE AFTER NO SUCCESSFUL WRITE, ref_id: {}, owner_id: {}", &file.reference_uuid, file.owner_uuid);
                            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file correctly"))
                        }
                    }
                }
            }
        }
    }

    Ok((StatusCode::CREATED, Json(response)))
}