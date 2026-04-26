use clap::Parser;
use uuid::Uuid;
use voip_sip::{
    Endpoint,
    message::{Request, headers, method, sip_uri},
    transaction::TsxPlugin,
};

use crate::cli::Cli;

mod cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let target = args.uri.parse()?;

    let endpoint = Endpoint::builder()
        .with_plugin(TsxPlugin::default())
        .with_udp_addr("0.0.0.0:9080")
        .build()
        .await?;

    let request = create_options_request(target);

    let client_tsx = endpoint.send_outgoing_request(request).await?;

    let _response = client_tsx.receive_final_response().await?;

    Ok(())
}

fn create_options_request(target: sip_uri::Uri) -> Request {
    let headers = build_headers(&target);
    let request = Request::with_headers(method::Method::Options, target, headers);
    request
}

fn build_headers(target: &sip_uri::Uri) -> headers::Headers {
    let hostip = voip_sip::utils::local_ip::get_local_ip_addr();

    let host = sip_uri::Host::IpAddr(hostip);
    let host_port = sip_uri::HostPort {
        host: host.clone(),
        port: Some(9080),
    };

    let uri = sip_uri::UriBuilder::new()
        .user(sip_uri::UserInfo {
            user: "sipc".to_owned(),
            pass: None,
        })
        .host(host_port.clone())
        .scheme(target.scheme)
        .build();

    let uri = sip_uri::NameAddr::new(uri);
    let uri = sip_uri::SipUri::NameAddr(uri);

    let via = headers::Via::new_udp(host_port.clone(), None, Some(headers::via::Rport(None)));
    let from = headers::From {
        uri: uri.clone(),
        tag: None,
        params: Default::default(),
    };
    let to = headers::To {
        uri: sip_uri::SipUri::Uri(target.clone()),
        tag: None,
        params: Default::default(),
    };
    let cseq = headers::CSeq::new(1, method::Method::Options);
    let call_id = headers::CallId::new(format!("{}@{}", Uuid::new_v4(), host));
    let max_forwards = headers::MaxForwards::new(70);
    let contact: headers::Contact = headers::Contact {
        uri,
        q: None,
        expires: None,
        param: Default::default(),
    };

    let headers = voip_sip::headers! {
        headers::Header::Via(via),
        headers::Header::From(from),
        headers::Header::To(to),
        headers::Header::CallId(call_id),
        headers::Header::Contact(contact),
        headers::Header::CSeq(cseq),
        headers::Header::MaxForwards(max_forwards)
    };

    headers
}
