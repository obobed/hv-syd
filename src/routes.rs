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
    const MARKDOWN_START: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/guide-start.html"));
    const MARKDOWN_END: &str = "</body>";

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

            let mut html = MARKDOWN_START.to_owned();
            html.reserve(estimated_html_length);

            pulldown_cmark::html::push_html(&mut html, parser);

            html.push_str(MARKDOWN_END);

            Ok(warp::reply::html(html).into_response())
        })
}
