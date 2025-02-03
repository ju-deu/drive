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
    filename: String
}

#[axum_macros::debug_handler]
pub async fn stream_upload(
    State(appstate): State<AppstateWrapper>,
    auth_user: Extension<AuthUser>,
    mut multipart: Multipart
) -> Result<(StatusCode, Json<Vec<Response>>), (StatusCode, &'static str)> {
    let appstate = appstate.0;
    let user = auth_user.0.0;

    let mut response: Vec<Response> = Vec::new();

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

        // mutable to later update file size
        let mut file = File::construct(
            None,
            filename.clone(),
            &user,
            0,
            &appstate
        ).await.ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to construct File"))?;

        // write to file in chunks
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Failed to get next chunk"))?
        {
            match file.write_chunk(chunk.as_ref()).await {
                Ok(_) => { file.size += chunk.len() },
                Err(_) => {
                    match file.delete_from_disk().await {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("FATAL: DANGLING FILE: {:?}; ERROR: {}", &file, e);
                        }
                    }
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file to disk"))
                }
            }// end match write_chunk
        }// end while let chunk

        // write file to db
        match file.write_to_db(&appstate).await {
            Ok(_) => {},
            Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to db")),
        }


        // add to response
        response.push( Response { reference_uuid: file.reference_uuid, filename: file.filename });

    }// end while let field

    Ok((StatusCode::CREATED, Json(response)))
}