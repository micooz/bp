use include_dir::{include_dir, Dir};
use tide::{http::mime, Request, Response};

static WEB_BUILD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web/build");

pub async fn assets(req: Request<()>) -> tide::Result {
    let index_html = WEB_BUILD_DIR.get_file("index.html").unwrap();
    let index_html = index_html.contents_utf8().unwrap();

    let res = match req.url().path() {
        "/" | "/index.html" => Response::builder(200).body(index_html).content_type(mime::HTML).build(),
        // path => Response::builder(200)
        //     .body(INDEX_JS_CONTENT)
        //     .content_type(mime::JAVASCRIPT)
        //     .build(),
        _ => Response::builder(404).build(),
    };

    Ok(res)
}
