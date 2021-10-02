use lazy_static;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveResult;
use trust_dns_resolver::lookup_ip::LookupIp;
use trust_dns_resolver::proto::op::{Header, MessageType};
use trust_dns_resolver::proto::rr::dnssec::SupportedAlgorithms;
use trust_dns_resolver::proto::rr::{dns_class::DNSClass, RData, Record};
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_server::authority::{AuthLookup, LookupObject, LookupRecords, MessageResponseBuilder};
use trust_dns_server::server::{RequestHandler, ServerFuture};

lazy_static::lazy_static! {
  static ref RESOLVER: TokioAsyncResolver = {
    // let mut opts = ResolverConfig::new();
    // opts.add_name_server(NameServerConfig {
        // socket_addr: SocketAddr::new(*ip, port),
        // protocol,
        // tls_dns_name: Some(tls_dns_name.clone()),
        // trust_nx_responses,
    // });
    TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap()
  };
}

pub async fn lookup(addr: &str) -> ResolveResult<LookupIp> {
    RESOLVER.lookup_ip(addr).await
}

pub async fn start_dns_server() {
    let socket = UdpSocket::bind("127.0.0.1:5355").await.unwrap();
    let mut future = ServerFuture::new(DnsServerHandle {});

    future.register_socket(socket);
    future.block_until_done().await.unwrap();
}

pub struct DnsServerHandle {}

impl RequestHandler for DnsServerHandle {
    type ResponseFuture = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

    fn handle_request<R: trust_dns_server::server::ResponseHandler>(
        &self,
        request: trust_dns_server::server::Request,
        mut response_handle: R,
    ) -> Self::ResponseFuture {
        let request_message = request.message;
        let queries = request_message.queries();

        let name = queries[0].name().to_string();
        dbg!(name);

        let mut response_header = Header::default();
        response_header.set_id(request_message.id());
        response_header.set_message_type(MessageType::Response);

        let answer = Record::new()
            .set_rdata(RData::A(Ipv4Addr::new(255, 255, 255, 255)))
            .set_dns_class(DNSClass::IN)
            .clone();

        let answers: Box<dyn LookupObject> = Box::new(AuthLookup::answers(
            LookupRecords::new(false, SupportedAlgorithms::all(), Arc::new(answer.into())),
            None,
        ));
        let name_servers: Box<dyn LookupObject> = Box::new(AuthLookup::default());
        let soa: Box<dyn LookupObject> = Box::new(AuthLookup::default());
        let additionals: Box<dyn LookupObject> = Box::new(AuthLookup::default());

        let response = MessageResponseBuilder::new(Some(request_message.raw_queries()));

        let result = response_handle.send_response(response.build(
            response_header,
            answers.iter(),
            name_servers.iter(),
            soa.iter(),
            additionals.iter(),
        ));

        if let Err(e) = result {
            log::error!("request error: {}", e);
        }

        Box::pin(async {})
    }
}
