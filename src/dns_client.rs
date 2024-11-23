use std::error::Error;
use std::net::SocketAddr;

use tokio::net::TcpStream;
use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::op::DnsResponse;
use trust_dns_client::proto::iocompat::AsyncIoTokioAsStd;
use trust_dns_client::rr::{DNSClass, Name, RecordType};
use trust_dns_client::tcp::TcpClientStream;

pub struct DnsClient {
    upstream: SocketAddr,
}

impl DnsClient {
    pub fn new(upstream: SocketAddr) -> Self {
        DnsClient { upstream }
    }

    pub async fn query(
        &self,
        name: Name,
        query_class: DNSClass,
        query_type: RecordType,
    ) -> Result<DnsResponse, Box<dyn Error>> {
        let (stream, sender) = TcpClientStream::<AsyncIoTokioAsStd<TcpStream>>::new(self.upstream);

        let (mut client, bg) = AsyncClient::new(stream, sender, None).await?;

        tokio::spawn(bg);

        Ok(client.query(name, query_class, query_type).await?)
    }
}
