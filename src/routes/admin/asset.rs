use std::{fs, path::PathBuf};

use log::trace;
use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{Error, PRIVATE_ASSET_FOLDER, PUBLIC_ASSET_FOLDER, routes::json_body};

#[derive(Serialize, Deserialize)]
struct CreateAssetBody {
    content: Vec<u8>,
    filename: PathBuf,
}

/// Create a public asset
pub fn create() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / "create")
        .and(warp::post())
        .and(json_body())
        .then(async |body: CreateAssetBody| -> Result<_, Error> {
            let filename = PathBuf::from(PUBLIC_ASSET_FOLDER)
                .join(body.filename.file_name().ok_or(Error::BadRequest)?);

            trace!(
                "create public asset (POST /api/admin/asset/{})",
                filename.file_name().unwrap().to_string_lossy()
            );

            tokio::fs::write(filename, body.content).await?;

            Ok(warp::reply())
        })
}

/// Delete a public asset
pub fn delete() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / ..)
        .and(warp::delete())
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| {
            let Ok(tail) = percent_encoding::percent_decode_str(tail.as_str()).decode_utf8() else {
                return Err(Error::NotFound);
            };
            trace!("delete public asset (DELETE /api/admin/asset/{})", tail);

            let mut path = PathBuf::from(PUBLIC_ASSET_FOLDER);

            for seg in tail.split('/') {
                if seg.starts_with("..") || seg.contains('\\') {
                    return Err(Error::NotFound);
                }
                path.push(seg);
            }

            if !path.exists() {
                return Err(Error::NotFound);
            }

            std::fs::remove_file(path)?;
            Ok(warp::reply())
        })
}

/// List private assets
pub fn list() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / "list")
        .and(warp::get())
        .map(|| -> Result<_, Error> {
            trace!("list public assets (GET /api/admin/list)");

            let out = fs::read_dir(PRIVATE_ASSET_FOLDER)?
                .map(|e| -> Result<_, Error> { Ok(e?.file_name().to_string_lossy().into_owned()) })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(warp::reply::json(&out))
        })
}

/// Get a private asset
pub fn get() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / ..)
        .and(warp::get())
        .and(warp::fs::dir(PRIVATE_ASSET_FOLDER))
        .map(|file: warp::filters::fs::File| {
            trace!(
                "get public asset (DELETE /api/admin/asset/{})",
                file.path().to_string_lossy()
            );

            file
        })
}
