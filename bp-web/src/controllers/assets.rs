use include_dir::{include_dir, Dir};
use tide::http::mime;

use crate::state::State;

static WEB_BUILD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web/build");

pub struct AssetsController;

impl AssetsController {
    pub async fn index(req: tide::Request<State>) -> tide::Result {
        let state = req.state();

        let index_html = WEB_BUILD_DIR.get_file("index.html").unwrap();
        let index_html_content = index_html.contents_utf8().unwrap();

        // replace __RUN_TYPE__ in index.html
        let index_html_content = index_html_content.replace("%RUN_TYPE%", &state.opts.run_type().to_string());

        Ok(tide::Response::builder(200)
            .body(index_html_content)
            .header("Cache-Control", "no-cache")
            .content_type(mime::HTML)
            .build())
    }

    pub async fn statics(req: tide::Request<State>) -> tide::Result {
        let path = req.url().path();

        let static_file = WEB_BUILD_DIR.get_file(&path[1..]); // ignore leading '/'
        if static_file.is_none() {
            return Ok(tide::Response::builder(404).build());
        }

        let static_file = static_file.unwrap();
        let static_file_ext = static_file
            .path()
            .extension()
            .map(|x| x.to_str().unwrap())
            .unwrap_or("");

        let content_type = match static_file_ext {
            "js" => mime::JAVASCRIPT,
            "css" => mime::CSS,
            _ => mime::ANY,
        };

        let static_file_content = static_file.contents_utf8().unwrap();

        Ok(tide::Response::builder(200)
            .body(static_file_content)
            .header("Cache-Control", "max-age=31536000")
            .content_type(content_type)
            .build())
    }
}
