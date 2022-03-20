use tide::http::headers;

use crate::{
    options::web::CryptoMethod,
    web::{
        common::{response::Response, state::State},
        utils::crypto::{Base64Crypto, Crypto, NoneCrypto},
    },
};

#[derive(Default)]
pub struct CryptoMiddleware;

impl CryptoMiddleware {
    fn get_crypto(method: CryptoMethod) -> Box<dyn Crypto<String> + Send> {
        match method {
            CryptoMethod::None => Box::new(NoneCrypto),
            CryptoMethod::Base64 => Box::new(Base64Crypto),
        }
    }
}

#[tide::utils::async_trait]
impl tide::Middleware<State> for CryptoMiddleware {
    async fn handle(&self, mut request: tide::Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let state = request.state();
        let path = request.url().path();

        if path == "/" || path == "/index.html" || path.contains("/static/") {
            return Ok(next.run(request).await);
        }

        let crypto = Self::get_crypto(state.opts.crypto.clone());

        // decode request body
        let req_body = request.body_string().await?;
        let decoded = crypto.decrypt(req_body)?;
        request.set_body(decoded);

        // encode response body
        let mut resp = next.run(request).await;

        // skip gzip-ed response
        if resp.header(headers::CONTENT_ENCODING).is_some() {
            return Ok(resp);
        }

        let status = resp.status().into();
        let body = resp.take_body();

        let resp_body = body.into_string().await.unwrap();
        let encoded = crypto.encrypt(resp_body)?;

        Ok(Response::base64(status, encoded))
    }
}
