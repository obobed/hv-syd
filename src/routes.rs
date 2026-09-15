use warp::{Filter, reply::Reply};

use crate::Error;

pub mod admin;
pub mod attendee;

const JSON_MAX_LENGTH_BYTES: u64 = 1024 * 64;

pub(crate) fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Copy
where
    T: serde::de::DeserializeOwned + Send,
{
    warp::body::content_length_limit(JSON_MAX_LENGTH_BYTES).and(warp::body::json::<T>())
}

pub fn public_assets() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path(crate::PUBLIC_ASSET_FOLDER).and(warp::fs::dir(crate::PUBLIC_ASSET_FOLDER))
}

pub fn guides() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    const MARKDOWN_OPTIONS: pulldown_cmark::Options = pulldown_cmark::Options::all();
    // <https://www.rfc-editor.org/info/rfc7763/>, <https://daringfireball.net/linked/2014/01/08/markdown-extension>
    const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "text"];

    // TODO: Possibly wrap the output of `html::push_html` with proper `<body>` tags,
    // add the `<!doctype html>`, possibly custom `<head>`, etc.

    warp::path(crate::GUIDES_FOLDER)
        .and(warp::fs::dir(crate::GUIDES_FOLDER))
        .then(async |f: warp::filters::fs::File| -> Result<_, Error> {
            let Some(extension) = f.path().extension() else {
                return Ok(f.into_response());
            };

            if !(MARKDOWN_EXTENSIONS.iter().any(|&s| extension == s)) {
                return Ok(f.into_response());
            }

            let markdown = tokio::fs::read_to_string(f.path()).await?;

            let parser = pulldown_cmark::Parser::new_ext(&markdown, MARKDOWN_OPTIONS);

            let estimated_html_length = markdown.len() * 3 / 2;
            let mut buf = String::with_capacity(estimated_html_length);

            pulldown_cmark::html::push_html(&mut buf, parser);

            Ok(warp::reply::html(buf).into_response())
        })
}
