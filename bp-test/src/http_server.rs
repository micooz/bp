use httpmock::prelude::*;
use std::net::SocketAddr;

const HTTP_SERVER_RESPONSE: &str = "some response text";

pub struct HttpServerContext {
    pub http_addr: SocketAddr,
    pub http_resp: &'static str,
}

pub fn run_http_mock_server(addr: Option<&str>) -> HttpServerContext {
    // Start a lightweight mock server.
    let server = if let Some(addr) = addr {
        MockServer::connect(addr)
    } else {
        MockServer::start()
    };

    // Create a mock on the server.
    server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200).body(HTTP_SERVER_RESPONSE);
    });

    HttpServerContext {
        http_addr: *server.address(),
        http_resp: HTTP_SERVER_RESPONSE,
    }
}
