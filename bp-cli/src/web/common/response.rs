use serde::Serialize;
use serde_json::Value;
use tide::{http::mime, Body};

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    success: bool,
    error_message: String,
    data: Value,
}

impl Response {
    pub fn success(data: Value) -> tide::Result {
        let body = Self {
            success: true,
            error_message: "".into(),
            data,
        };
        Ok(Self::json(200, body))
    }

    pub fn error(status: u16, message: &str) -> tide::Result {
        Ok(Self::plain(status, message))
    }

    pub fn json<T: Into<tide::Body>>(status: u16, body: T) -> tide::Response {
        Self::build_response(status, body, mime::JSON)
    }

    pub fn plain<T: Into<tide::Body>>(status: u16, body: T) -> tide::Response {
        Self::build_response(status, body, mime::PLAIN)
    }

    pub fn base64<T: Into<tide::Body>>(status: u16, body: T) -> tide::Response {
        Self::build_response(status, body, mime::BYTE_STREAM)
    }

    fn build_response<T: Into<tide::Body>>(status: u16, body: T, content_type: mime::Mime) -> tide::Response {
        tide::Response::builder(status)
            .body(body)
            .content_type(content_type)
            .build()
    }
}

impl From<Response> for tide::Body {
    fn from(resp: Response) -> Self {
        Body::from_json(&resp).unwrap()
    }
}
