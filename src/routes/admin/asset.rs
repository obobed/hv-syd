use std::{fs, path::PathBuf};

use log::trace;
use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{Error, PRIVATE_ASSET_FOLDER, PUBLIC_ASSET_FOLDER};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CreateAssetBody {
    #[cfg_attr(feature = "openapi", schema(example = json!([0, 1, 2, 3])))]
    content: Vec<u8>,
}

/// Create a public asset
///
/// Accepts file content and writes it to a file on disk, in the "assets/"
/// folder. Note that directories are not supported, and the payload requires
/// a *filename*, not a *path*.
#[cfg_attr(feature = "openapi", utoipa::path(
    put,
    path = "/asset/{path}",
    params((
        "path" = String,
        Path,
        description = "Path of file to create, relative to the public asset folder",
        allow_reserved = true
    )),
    request_body = CreateAssetBody,
    responses(
        (status = 200, description = "Asset was created."),
        (status = 404, description = "URL decoding failed, probably because the path was not valid UTF-8."),
        (status = 400, description = "The given path was a directory (ends in `/`)."),
        (status = 500, description = "An internal IO error occured.")
    ),
))]
pub fn create() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / ..)
        .and(warp::put())
        .and(warp::body::json()) // <- `crate::routes::json_body` has a 64KiB limit
        .and(warp::path::tail())
        .then(
            async |body: CreateAssetBody, tail: warp::path::Tail| -> Result<_, Error> {
                let Ok(tail) = percent_encoding::percent_decode_str(tail.as_str()).decode_utf8()
                else {
                    return Err(Error::NotFound);
                };

                let path = sanitise_path(PathBuf::from(PUBLIC_ASSET_FOLDER), &tail)?;

                // `Path::has_trailing_sep` is still unstable so,,,
                if path
                    .as_os_str()
                    .as_encoded_bytes()
                    .last()
                    .is_some_and(|&b| std::path::is_separator(b as char))
                {
                    return Err(Error::BadRequest);
                }

                trace!(
                    "create public asset (POST /api/admin/asset/{})",
                    path.to_string_lossy()
                );

                // NOTE: Keep this sync! This makes sure the file can actually be created ;-;
                fs::create_dir_all(path.parent().ok_or(Error::NotFound)?)?;

                tokio::fs::write(path, body.content).await?;

                Ok(warp::reply())
            },
        )
}

/// Delete a public asset
///
/// Given a path, deletes the *file* located there. Note that directories are
/// not supported.
#[cfg_attr(feature = "openapi", utoipa::path(
    delete,
    path = "/asset/{path}",
    params((
        "path" = String,
        Path,
        description = "Path of file to delete, relative to the public asset folder",
        allow_reserved = true
    )),
    responses(
        (status = 200, description = "Asset was deleted."),
        (status = 404, description = "No file found to delete."),
        (status = 500, description = "An internal IO error occured.")
    ),
))]
pub fn delete() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / ..)
        .and(warp::delete())
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| {
            let Ok(tail) = percent_encoding::percent_decode_str(tail.as_str()).decode_utf8() else {
                return Err(Error::NotFound);
            };
            trace!("delete public asset (DELETE /api/admin/asset/{})", tail);

            let path = sanitise_path(PathBuf::from(PUBLIC_ASSET_FOLDER), &tail)?;
            if !path.exists() {
                return Err(Error::NotFound);
            }

            std::fs::remove_file(path)?;
            Ok(warp::reply())
        })
}

/// List private assets
///
/// Lists all files currently in the `/assets` folder.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/asset/list",
    responses(
        (status = 200, body = Vec<String>, description = "Files were listed."),
        (status = 404, description = "The `/assets` folder could not be found on the server."),
        (status = 500, description = "An internal IO error occured.")
    ),
))]
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

#[cfg(feature = "openapi")]
#[derive(utoipa::ToSchema)]
#[schema(content_encoding = "binary")]
#[expect(dead_code)]
struct File(String);

/// Get a private asset
///
/// Given a path, get the *file* located there. Note that directories are
/// not supported.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/asset/{path}",
    params((
        "path" = String,
        Path,
        description = "Path of file, relative to the public asset folder",
        allow_reserved = true
    )),
    responses(
        (
            status = 200,
            body = File,
            content_type = "*/*",
            description = "File contents. Note that content type is estimated for each file."
        ),
        (status = 404, description = "No file found to delete."),
        (status = 500, description = "An internal IO error occured.")
    ),
))]
pub fn get() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset" / ..)
        .and(warp::get())
        .and(warp::fs::dir(PRIVATE_ASSET_FOLDER))
        .map(|file: warp::filters::fs::File| {
            trace!(
                "get public asset (GET /api/admin/asset/{})",
                file.path().to_string_lossy()
            );

            file
        })
}

fn sanitise_path(mut head: PathBuf, tail: &str) -> Result<PathBuf, Error> {
    for seg in tail.split('/') {
        if seg.starts_with("..") || seg.contains('\\') {
            return Err(Error::NotFound);
        }
        head.push(seg);
    }

    Ok(head)
}

#[cfg(test)]
mod test {
    use crate::{GUIDES_FOLDER, TOKENS_STORE_FOLDER};

    use super::{PRIVATE_ASSET_FOLDER, PUBLIC_ASSET_FOLDER, create, delete, get, list};
    use rand::{
        distr::{Alphanumeric, SampleString as _},
        rng,
    };
    use serde_json::json;
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
        sync::{Mutex, Once},
    };
    use warp::{http, test::request};

    const TEST_FILE: (&[u8], &str) = (b"woof woof :3", "woof.txt");

    static PRIVATE_DIR_LOCK: Mutex<u8> = Mutex::new(0);
    static CREATE_DIRS: Once = Once::new();

    /// Macro for simplifying setup operations that could fail
    macro_rules! setup_assert {
        ($lhs:expr) => {
            if !($lhs) {
                eprintln!("test failed due to system error, should retry.");
                panic!();
            }
        };
    }

    fn make_test_path(base: impl AsRef<std::ffi::OsStr>) -> PathBuf {
        PathBuf::from(&base).join(Alphanumeric.sample_string(&mut rng(), 6))
    }

    fn initialise_folders() {
        setup_assert!(fs::create_dir_all(PUBLIC_ASSET_FOLDER).is_ok());
        setup_assert!(fs::create_dir_all(PRIVATE_ASSET_FOLDER).is_ok());
        setup_assert!(fs::create_dir_all(GUIDES_FOLDER).is_ok());
        setup_assert!(fs::create_dir_all(TOKENS_STORE_FOLDER).is_ok());
    }

    #[tokio::test]
    async fn test_create() {
        CREATE_DIRS.call_once(initialise_folders);

        let test_file_path = make_test_path(PUBLIC_ASSET_FOLDER);
        let payload = HashMap::from([("content", json!(TEST_FILE.0))]);

        _ = fs::remove_file(&test_file_path);
        setup_assert!(!test_file_path.exists());

        let res = request()
            .path(&format!(
                "/asset/{}",
                test_file_path.file_name().unwrap().to_string_lossy()
            ))
            .method("PUT")
            .json(&payload)
            .reply(&create())
            .await;

        assert_eq!(res.status(), http::StatusCode::OK);
        assert!(test_file_path.exists());

        setup_assert!(fs::remove_file(test_file_path).is_ok())
    }

    #[tokio::test]
    async fn test_delete() {
        CREATE_DIRS.call_once(initialise_folders);

        let test_file_path = make_test_path(PUBLIC_ASSET_FOLDER);

        setup_assert!(fs::write(&test_file_path, TEST_FILE.0).is_ok());
        setup_assert!(test_file_path.exists());

        let res = request()
            .path(&format!(
                "/asset/{}",
                test_file_path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap()
            ))
            .method("DELETE")
            .reply(&delete())
            .await;

        assert_eq!(res.status(), http::StatusCode::OK);
        assert!(!test_file_path.exists());
    }

    #[tokio::test]
    async fn test_get() {
        CREATE_DIRS.call_once(initialise_folders);

        let test_file_path = make_test_path(PRIVATE_ASSET_FOLDER);

        let guard = PRIVATE_DIR_LOCK.lock().unwrap();

        setup_assert!(fs::write(&test_file_path, TEST_FILE.0).is_ok());
        setup_assert!(test_file_path.exists());

        let res = request()
            .path(&format!(
                "/asset/{}",
                test_file_path.file_name().unwrap().to_string_lossy()
            ))
            .method("GET")
            .reply(&get())
            .await;

        setup_assert!(fs::remove_file(&test_file_path).is_ok());

        drop(guard);

        assert_eq!(res.status(), http::StatusCode::OK);
        assert_eq!(res.into_body(), TEST_FILE.0);
    }

    #[tokio::test]
    async fn test_list() {
        CREATE_DIRS.call_once(initialise_folders);

        let test_file_paths = (0..5)
            .map(|_| make_test_path(PRIVATE_ASSET_FOLDER))
            .collect::<Vec<_>>();

        let guard = PRIVATE_DIR_LOCK.lock().unwrap();

        for p in test_file_paths.iter() {
            setup_assert!(fs::write(&p, TEST_FILE.0).is_ok());
            setup_assert!(Path::new(&p).exists());
        }

        let mut res = request()
            .path("/asset/list")
            .method("GET")
            .reply(&list())
            .await
            .map(|b| serde_json::from_slice::<Vec<String>>(&b).unwrap());

        res.body_mut().sort();

        for p in test_file_paths.iter() {
            setup_assert!(fs::remove_file(&p).is_ok());
            setup_assert!(!Path::new(&p).exists());
        }

        drop(guard);

        let mut rhs = test_file_paths
            .into_iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        rhs.sort();

        assert_eq!(res.status(), http::StatusCode::OK);
        assert_eq!(res.into_body(), rhs);
    }
}
