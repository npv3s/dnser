use std::net::SocketAddr;

use clap::Parser;
use ipnet::Ipv4Net;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    /// Listening UDP address.
    #[clap(long, default_value = "0.0.0.0:53")]
    pub udp: Vec<SocketAddr>,

    /// Listening TCP address.
    #[clap(long)]
    pub tcp: Vec<SocketAddr>,

    /// Upstream DNS server address.
    #[clap(long, short, default_value = "8.8.8.8:53")]
    pub upstream: SocketAddr,

    /// Virtual adresses subnet for routing.
    #[clap(long, short, default_value = "192.168.128.0/17")]
    pub routed_subnet: Ipv4Net,

    /// Path to the list of redirected domains.
    #[clap(long, short, default_value = "domains.txt")]
    pub domains_list: String,
}
