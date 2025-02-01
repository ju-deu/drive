use std::io::Read;
use std::path::Path;
use axum::Extension;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio_stream::Stream;
use crate::models::appstate::AppstateWrapper;
use crate::models::file::File;
use crate::models::user::AuthUser;

pub async fn stream_upload(
    State(appstate): State<AppstateWrapper>,
    auth_user: Extension<AuthUser>,
    mut multipart: Multipart
) -> Result<StatusCode, (StatusCode, String)> {
    let appstate = appstate.0;
    let user = auth_user.0.0;

    while let Ok(Some(mut field)) = multipart.next_field().await {


        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|err| (StatusCode::BAD_REQUEST, format!("Failed to get next chunk: {}",err)))?
        {
            println!("received {} bytes", chunk.len());

            let mut file = tokio::fs::File::options().append(true)
                .open(r"/home/fabi/RustroverProjects/drive/debug.txt").await.expect("Failed to open File");

            file.write(&*chunk).await.expect("TODO: panic message");

        }
        let len = field.bytes().await.unwrap().len();
        println!("In memory len: {}", len);
    }

    Ok(StatusCode::CREATED)
}