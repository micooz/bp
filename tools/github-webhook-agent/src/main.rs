use clap::Parser;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use std::net::SocketAddr;
use vial::prelude::*;

const SECRET_TOKEN_ENV_NAME: &str = "SECRET_TOKEN";
const SIGNATURE_HEADER_NAME: &str = "X-Hub-Signature-256";

struct GlobalState {
    secret_token: String,
}

#[derive(Parser, Debug)]
#[clap(name = "bp", version = clap::crate_version!())]
struct Options {
    #[clap(short, long)]
    host: String,

    #[clap(short, long)]
    port: u16,
}

impl Options {
    pub fn addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }
}

fn main() {
    let (_, secret_token) = std::env::vars()
        .find(|(key, value)| key == SECRET_TOKEN_ENV_NAME && !value.is_empty())
        .unwrap_or_else(|| panic!("env var {} is not set", SECRET_TOKEN_ENV_NAME));

    let opts: Options = clap::Parser::parse();

    vial::use_state!(GlobalState { secret_token });
    vial::run!(opts.addr()).unwrap();
}

routes! {
    POST "/" => handle_request;
}

fn handle_request(req: Request) -> Response {
    let global_state = req.state::<GlobalState>();

    let signature = req.header(SIGNATURE_HEADER_NAME);

    if signature.is_none() {
        return Response::from(403).with_body(format!("{} header not provided", SIGNATURE_HEADER_NAME));
    }

    let signature = signature.unwrap().to_string();
    let signature = signature.trim_start_matches("sha256=");
    let signature = hex::decode(signature);

    if let Err(err) = signature {
        return Response::from(403).with_body(err.to_string());
    }

    let signature = signature.unwrap();
    let body = req.body();

    if let Err(err) = signature_check(
        body.as_bytes(),
        signature.as_slice(),
        global_state.secret_token.as_bytes(),
    ) {
        return Response::from(403).with_body(err);
    }

    // TODO: publish a notify with req.body

    Response::from(200)
}

fn signature_check(content: &[u8], input_signature: &[u8], key: &[u8]) -> Result<(), String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| "fail to init HMAC")?;
    mac.update(content);
    mac.verify(input_signature).map_err(|_| "signature mismatch")?;
    Ok(())
}

#[test]
fn test_signature_check() {
    assert!(signature_check(b"foo", b"xxx", b"key").is_err());
    assert!(signature_check(
        b"foo",
        hex::decode("6ea1d9f5e93a8f3ade026261ffe5d72a1c90804ed94404a69892a163b8a35497")
            .unwrap()
            .as_slice(),
        b"key"
    )
    .is_ok());
}
