use clap::Parser;
use rssip::{
    Endpoint, IncomingResponse, OutgoingRequest,
    endpoint::{self, ToTake},
    message::{Request, headers, method, uri},
    transaction::{ClientTransaction, TsxPlugin},
};
use tracing::Level;
use tracing_subscriber::fmt::time::ChronoLocal;
use uuid::Uuid;

use crate::cli::Cli;

mod cli;

struct Logger;

#[async_trait::async_trait]
impl endpoint::Plugin for Logger {
    fn name(&self) -> &'static str {
        "logger"
    }

    async fn on_outgoing_request(&self, req: &mut OutgoingRequest) {
        println!("{}{}", req.req_line, req.headers);
    }
    async fn on_incoming_response(&self, res: ToTake<'_, IncomingResponse>, _endpoint: &Endpoint) {
        println!("{}{}", res.status_line, res.headers);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("rssip=trace")
        .with_timer(ChronoLocal::new(String::from("%H:%M:%S%.3f")))
        .init();

    let args = Cli::parse();
    let target = args.uri.parse()?;

    let endpoint = create_endpoint().await?;
    let options_request = build_options_request(target);

    let client_tsx = ClientTransaction::send_request(options_request, endpoint).await?;

    let _ = client_tsx.receive_final_response().await?;

    Ok(())
}

async fn create_endpoint() -> rssip::Result<Endpoint> {
    Endpoint::builder()
        .with_plugin(Logger)
        .with_plugin(TsxPlugin::default())
        .with_udp_addr("0.0.0.0:9080")
        .build()
        .await
}

fn build_options_request(target: uri::Uri) -> Request {
    let headers = build_headers(&target);
    let request = Request::with_headers(method::SipMethod::Options, target, headers);
    request
}

fn build_headers(target: &uri::Uri) -> headers::Headers {
    use headers::Header;
    let hostip = rssip::utils::local_ip::get_local_ip_addr();

    let host = uri::Host::IpAddr(hostip);

    let host_port = uri::HostPort {
        host: host.clone(),
        port: Some(9080),
    };

    let uri = uri::Uri {
        scheme: target.scheme,
        user: Some(uri::UserInfo {
            user: "sipc".to_owned(),
            pass: None,
        }),
        host_port: host_port.clone(),
        ..Default::default()
    };
    let uri = uri::SipUri::NameAddr(uri::NameAddr { display: None, uri });

    let via = headers::Via::new_udp(host_port, None, Some(headers::via::Rport(None)));
    let from = headers::From {
        uri: uri,
        tag: Some(rssip::utils::generate_tag()),
        params: Default::default(),
    };
    let to = headers::To {
        uri: uri::SipUri::Uri(target.clone()),
        tag: None,
        params: Default::default(),
    };
    let cseq = headers::CSeq::new(1, method::SipMethod::Options);
    let call_id = headers::CallId::new(format!("{}@{}", Uuid::new_v4(), host));
    let max_forwards = headers::MaxForwards::new(70);

    let headers = rssip::headers! {
        Header::Via(via),
        Header::From(from),
        Header::To(to),
        Header::CallId(call_id),
        Header::CSeq(cseq),
        Header::MaxForwards(max_forwards),
    };

    headers
}
