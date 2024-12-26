use std::error::Error;
use std::net::SocketAddr;

use hickory_client::client::{AsyncClient, ClientHandle};
use hickory_client::op::DnsResponse;
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::rr::{DNSClass, Name, RecordType};
use hickory_client::tcp::TcpClientStream;
use tokio::net::TcpStream;

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
