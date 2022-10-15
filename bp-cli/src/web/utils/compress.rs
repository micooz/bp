use async_compression::futures::bufread::GzipEncoder;
use async_std::io::BufReader;
use tide::{
    http::{
        conditional::Vary,
        content::{AcceptEncoding, Encoding},
        headers, Method,
    },
    Body, Request, Response,
};

pub fn gzip<State: Clone + Send + Sync + 'static>(req: &Request<State>, mut res: Response) -> tide::Result {
    let accepts = AcceptEncoding::from_headers(req)?;

    // skip HEAD and no accepts headers requests
    if req.method() == Method::Head || accepts.is_none() {
        return Ok(res);
    }

    let mut accepts = accepts.unwrap();

    // set response Vary
    let mut vary = Vary::new();
    vary.push(headers::ACCEPT_ENCODING)?;
    vary.apply(&mut res);

    // set response Content-Encoding
    let encoding = accepts.negotiate(&[Encoding::Gzip])?;
    encoding.apply(&mut res);

    // set compressed response body
    let raw_body = res.take_body();
    let compressed_body = Body::from_reader(BufReader::new(GzipEncoder::new(raw_body)), None);
    res.set_body(compressed_body);

    // remove response Content-Length
    res.remove_header(headers::CONTENT_LENGTH);

    Ok(res)
}
