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

// Just gimme a const function :c
const MARKDOWN_OPTIONS: markdown::Options = markdown::Options {
    parse: markdown::ParseOptions {
        constructs: markdown::Constructs {
            attention: true,
            autolink: true,
            block_quote: true,
            character_escape: true,
            character_reference: true,
            code_indented: true,
            code_fenced: true,
            code_text: true,
            definition: true,
            frontmatter: false,
            gfm_autolink_literal: true,
            gfm_footnote_definition: true,
            gfm_label_start_footnote: true,
            gfm_strikethrough: true,
            gfm_table: true,
            gfm_task_list_item: true,
            hard_break_escape: true,
            hard_break_trailing: true,
            heading_atx: true,
            heading_setext: true,
            html_flow: true,
            html_text: true,
            label_start_image: true,
            label_start_link: true,
            label_end: true,
            list_item: true,
            math_flow: false,
            math_text: false,
            mdx_esm: false,
            mdx_expression_flow: false,
            mdx_expression_text: false,
            mdx_jsx_flow: false,
            mdx_jsx_text: false,
            thematic_break: true,
        },
        gfm_strikethrough_single_tilde: true,
        math_text_single_dollar: true,
        mdx_expression_parse: None,
        mdx_esm_parse: None,
    },
    compile: markdown::CompileOptions {
        allow_any_img_src: true,
        allow_dangerous_html: true,
        allow_dangerous_protocol: false,
        default_line_ending: markdown::LineEnding::LineFeed,
        gfm_footnote_back_label: None,
        gfm_footnote_clobber_prefix: None,
        gfm_footnote_label_attributes: None,
        gfm_footnote_label_tag_name: None,
        gfm_footnote_label: None,
        gfm_task_list_item_checkable: false,
        gfm_tagfilter: false,
    },
};

pub fn guides() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path(crate::GUIDES_FOLDER)
        .and(warp::fs::dir(crate::GUIDES_FOLDER))
        .then(async |f: warp::filters::fs::File| -> Result<_, Error> {
            let Some(extension) = f.path().extension() else {
                return Ok(f.into_response());
            };

            if !(extension == "md") {
                return Ok(f.into_response());
            }

            // This is two whole newly-allocated strings, is there a better way?
            let contents = tokio::fs::read_to_string(f.path()).await?;
            // GFM apparently cannot error, so unwrapping.
            let content = markdown::to_html_with_options(&contents, &MARKDOWN_OPTIONS).unwrap();

            Ok(warp::reply::html(content).into_response())
        })
}
