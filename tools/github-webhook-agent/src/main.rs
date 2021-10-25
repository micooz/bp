mod config;
mod constants;
mod context;
mod options;
mod utils;

use crate::constants::{GITHUB_SECRET_TOKEN_ENV_NAME, SIGNATURE_HEADER_NAME, WEBHOOK_SECRET_TOKEN_ENV_NAME};
use config::Config;
use constants::TMP_DATA_FILE;
use context::Context;
use hmac::{Hmac, Mac, NewMac};
use options::Options;
use serde_json::{Map, Value};
use sha2::Sha256;
use std::io::Write;
use vial::prelude::*;

struct GlobalState {
    env: Map<String, Value>,
    config: Config,
}

fn main() {
    // check env var
    let vars = vec![WEBHOOK_SECRET_TOKEN_ENV_NAME, GITHUB_SECRET_TOKEN_ENV_NAME];
    let mut env = Map::with_capacity(2);

    for name in vars {
        let (_, value) = std::env::vars()
            .find(|(key, value)| key == name && !value.is_empty())
            .unwrap_or_else(|| panic!("env var {} is not set", name));

        env.insert(name.to_string(), Value::String(value));
    }

    // parse options
    let opts: Options = clap::Parser::parse();

    // load yaml config
    let config = Config::from_file(&opts.config).expect("fail to load config file");

    vial::use_state!(GlobalState { env, config });
    vial::run!(opts.bind).unwrap();
}

routes! {
    POST "/" => handle_request;
}

fn handle_request(req: Request) -> Response {
    let global_state = req.state::<GlobalState>();
    let env = &global_state.env;

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

    let checked = signature_check(
        body.as_bytes(),
        signature.as_slice(),
        env[WEBHOOK_SECRET_TOKEN_ENV_NAME].as_str().unwrap().as_bytes(),
    );

    if let Err(err) = checked {
        return Response::from(403).with_body(err);
    }

    // parse json
    let body = serde_json::from_str(req.body());
    let body = body.as_ref();

    if let Err(err) = body {
        return Response::from(500).with_body(err.to_string());
    }

    let body: &Value = body.unwrap();

    // store body as json file
    let json = serde_json::to_string(body).unwrap();

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(TMP_DATA_FILE)
        .unwrap();

    file.write_all(json.as_bytes()).unwrap();
    file.flush().unwrap();

    // find & execute rule
    let rule = global_state.config.try_match(&req, body);

    if rule.is_none() {
        return Response::from(404).with_body("No rule matched");
    }

    let rule = rule.unwrap();
    let env = serde_json::Value::Object(env.clone());

    let ctx = Context {
        body: Some(body),
        secrets: Some(&env),
    };

    let stdout = rule.run(ctx).unwrap();
    println!("{}", stdout);

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
