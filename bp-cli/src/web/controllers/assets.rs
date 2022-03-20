use include_dir::{include_dir, Dir};
use tide::http::{headers, mime};

use crate::web::{common::state::State, utils::compress};

static WEB_BUILD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../bp-web/build");

pub struct AssetsController;

impl AssetsController {
    pub async fn index(req: tide::Request<State>) -> tide::Result {
        let state = req.state();

        let index_html = WEB_BUILD_DIR.get_file("index.html").unwrap();
        let index_html_content = index_html.contents_utf8().unwrap();

        // replace placeholders in index.html
        let index_html_content = index_html_content
            .replace("%REACT_APP_VERSION%", env!("CARGO_PKG_VERSION"))
            // TODO: find a way to obtain rustc version
            .replace("%REACT_APP_RUST_VERSION%", "")
            .replace("%REACT_APP_RUN_TYPE%", &state.opts.run_type().to_string())
            .replace("%REACT_APP_CRYPTO_METHOD%", &state.opts.crypto.to_string());

        Ok(tide::Response::builder(200)
            .body(index_html_content.as_bytes())
            .header(headers::CACHE_CONTROL, "no-store")
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

        let res = tide::Response::builder(200)
            .body(static_file_content)
            .header(headers::CACHE_CONTROL, "max-age=31536000")
            .content_type(content_type)
            .build();

        compress::gzip(&req, res)
    }
}
