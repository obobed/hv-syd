use std::{
    fs,
    path::{Path, PathBuf},
};

use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, query_file};
use warp::Filter;

use crate::{Error, PRIVATE_ASSET_FOLDER};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct UploadBody {
    #[cfg_attr(feature = "openapi", schema(example = json!([0, 1, 2, 3])))]
    content: Vec<u8>,
    filename: String,
}

/// Create a public asset
///
/// Accepts file content and writes it to a file on disk. Directories are not
/// supported.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/upload",
    request_body = UploadBody,
    responses(
        (status = 200, body = String, description = "Asset was created, returns path to asset (relative to host)"),
        (status = 400, description = "The given path was a directory (ends in `/`)."),
        (status = 404, description = "Attendee didn't have a team."),
        (status = 408, description = "Asset with same name already exists."),
        (status = 500, description = "An internal IO error occured.")
    ),
))]
pub fn upload(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("upload")
        .and(warp::post())
        .and(warp::cookie("attendee-id"))
        .and(warp::body::json()) // <- `crate::routes::json_body` has a 64KiB limit
        .map(|id, body| (db.clone(), id, body))
        .untuple_one()
        .then(
            async |db, id: String, body: UploadBody| -> Result<_, Error> {
                trace!("create private asset (POST /api/v1/upload)");

                let team = query_file!("sql/get-team-name.sql", &id)
                    .fetch_one(&db)
                    .await?;

                let team_name = team.name.ok_or(Error::NotFound)?;

                let mut path_on_disk = PathBuf::from(PRIVATE_ASSET_FOLDER);

                let relative_path = format!(
                    "{}/{}",
                    team_name,
                    Path::new(&body.filename)
                        .file_name()
                        .ok_or(Error::BadRequest)?
                        .display()
                );

                path_on_disk.push(&relative_path);

                if path_on_disk.exists() {
                    return Err(Error::Conflict);
                }

                // NOTE: Keep this sync! This makes sure the file can actually be created ;-;
                fs::create_dir_all(path_on_disk.parent().ok_or(Error::NotFound)?)?;

                tokio::fs::write(&path_on_disk, body.content).await?;

                Ok(format!("/api/admin/asset/{relative_path}"))
            },
        )
}
