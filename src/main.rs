use clap::Parser;
use hickory_server::ServerFuture;
use tokio::net::{TcpListener, UdpSocket};
use tokio::signal;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use handler::Handler;
use options::Options;

use crate::dns_client::DnsClient;
use crate::domain_filter::DomainFilter;
use crate::nat_router::NatRouter;

mod dns_client;
mod domain_filter;
mod handler;
mod nat_router;
mod options;

const TCP_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let options = Options::parse();
    let domain_filter = match DomainFilter::from_file(options.domains_list.as_str()) {
        Ok(domain_filter) => domain_filter,
        Err(err) => {
            error!("couldn't read domains list file: {}", err);
            return;
        }
    };
    let nat_router = NatRouter::new(options.routed_subnet);
    let dns_client = DnsClient::new(options.upstream);
    let handler = Handler::new(domain_filter, nat_router, dns_client);

    let mut server = ServerFuture::new(handler);

    for udp in &options.udp {
        match UdpSocket::bind(&udp).await {
            Ok(socket) => server.register_socket(socket),
            Err(err) => {
                error!("failed to bind udp {} {}", udp, err);
                return;
            }
        }
    }

    for tcp in &options.tcp {
        match TcpListener::bind(&tcp).await {
            Ok(socket) => server.register_listener(socket, TCP_TIMEOUT),
            Err(err) => {
                error!("failed to bind tcp {} {}", tcp, err);
                return;
            }
        }
    }

    tokio::spawn(async move {
        match server.block_until_done().await {
            Ok(_) => (),
            Err(err) => {
                error!("dns server thread failed: {}", err);
                return;
            }
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {
            sleep(Duration::from_millis(100)).await;
            info!("shutting down");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
