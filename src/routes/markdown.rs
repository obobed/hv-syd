use std::sync::LazyLock;

use warp::reply::Reply as _;

use crate::Error;

const MARKDOWN_OPTIONS: LazyLock<pulldown_cmark::Options> = LazyLock::new(|| {
    use pulldown_cmark::Options;
    Options::all()
        & !Options::ENABLE_SMART_PUNCTUATION
        & !Options::ENABLE_OLD_FOOTNOTES
        & !Options::ENABLE_MATH
});
// <https://www.rfc-editor.org/info/rfc7763/>, <https://daringfireball.net/linked/2014/01/08/markdown-extension>
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "text"];
const MARKDOWN_START: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/guide-start.html"));
const MARKDOWN_END: &str = "</body>";

pub async fn handler(f: warp::filters::fs::File) -> Result<impl warp::Reply, Error> {
    let Some(extension) = f.path().extension() else {
        return Ok(f.into_response());
    };

    if !(MARKDOWN_EXTENSIONS.iter().any(|&s| extension == s)) {
        return Ok(f.into_response());
    }

    let markdown = tokio::fs::read_to_string(f.path()).await?;

    let iter = ext::with_alerts(pulldown_cmark::Parser::new_ext(
        &markdown,
        *MARKDOWN_OPTIONS,
    ));

    let estimated_html_length = markdown.len() * 3 / 2;

    let mut html = MARKDOWN_START.to_owned();
    html.reserve(estimated_html_length);

    pulldown_cmark::html::push_html(&mut html, iter);

    html.push_str(MARKDOWN_END);

    Ok(warp::reply::html(html).into_response())
}

mod ext {
    use pulldown_cmark::{BlockQuoteKind, CowStr, Event, Tag, TagEnd};

    struct WithAlerts<'a, I>
    where
        I: Iterator<Item = pulldown_cmark::Event<'a>>,
    {
        inner: I,
    }

    impl<'a, I> Iterator for WithAlerts<'a, I>
    where
        I: Iterator<Item = pulldown_cmark::Event<'a>>,
    {
        type Item = pulldown_cmark::Event<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            let event = self.inner.next();

            let Some(Event::Start(Tag::BlockQuote(Some(kind)))) = event else {
                return event;
            };

            let (class_str, title, icon) = match kind {
                BlockQuoteKind::Note => (
                    " class=\"markdown-alert markdown-alert-note\"",
                    "Note",
                    include_str!("../../assets/note.svg"),
                ),
                BlockQuoteKind::Tip => (
                    " class=\"markdown-alert-tip\"",
                    "Tip",
                    include_str!("../../assets/tip.svg"),
                ),
                BlockQuoteKind::Important => (
                    " class=\"markdown-alert markdown-alert-important\"",
                    "Important",
                    include_str!("../../assets/important.svg"),
                ),
                BlockQuoteKind::Warning => (
                    " class=\"markdown-alert markdown-alert-warning\"",
                    "Warning",
                    include_str!("../../assets/warning.svg"),
                ),
                BlockQuoteKind::Caution => (
                    " class=\"markdown-alert markdown-alert-caution\"",
                    "Caution",
                    include_str!("../../assets/caution.svg"),
                ),
            };

            let mut quoted_content = format!(
                "<blockquote{}>\n<p class=\"markdown-alert-title\">{}\n{}</p>",
                class_str, icon, title
            );

            let quoted_iter = self.inner.by_ref().take_while(|inner| match inner {
                Event::End(TagEnd::BlockQuote(Some(end))) if end == &kind => return false,
                _ => return true,
            });

            pulldown_cmark::html::push_html(&mut quoted_content, quoted_iter);

            quoted_content.push_str("</blockquote>\n");

            Some(Event::Html(CowStr::Boxed(quoted_content.into_boxed_str())))
        }
    }

    /// Returns an iterator that renders GFM alerts with the correct icons and
    /// titles.
    pub fn with_alerts<'a, I>(iter: I) -> impl Iterator<Item = Event<'a>>
    where
        I: Iterator<Item = Event<'a>>,
    {
        WithAlerts { inner: iter }
    }
}
