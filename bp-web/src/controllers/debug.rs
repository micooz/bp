pub struct DebugController;

impl DebugController {
    pub async fn index(req: tide::Request<()>) -> tide::Result {
        log::debug!("method = {}, url = {}", req.method(), req.url());
        Ok(tide::Response::builder(404).build())
    }
}
