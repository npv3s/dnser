use std::error::Error;

use hickory_server::{
    authority::MessageResponseBuilder,
    proto::op::{Header, ResponseCode},
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};
use tracing::*;

use crate::dns_client::DnsClient;
use crate::domain_filter::DomainFilter;
use crate::nat_router::NatRouter;

pub struct Handler {
    domain_filter: DomainFilter,
    nat_router: NatRouter,
    dns_client: DnsClient,
}

impl Handler {
    pub fn new(domain_filter: DomainFilter, nat_router: NatRouter, dns_client: DnsClient) -> Self {
        Handler {
            domain_filter,
            nat_router,
            dns_client,
        }
    }

    async fn do_handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Box<dyn Error>> {
        let response = self
            .dns_client
            .query(
                request.query().name().into(),
                request.query().query_class(),
                request.query().query_type(),
            )
            .await
            .map_err(|err| {
                error!("failed to get dns response from upstream: {}", err);
                err
            })?;

        let mut answers = Vec::from(response.answers());

        if self.domain_filter.check(request.query().name()) {
            for record in answers.iter_mut() {
                if let Some(rdata) = record.data_mut() {
                    if let Some(a) = rdata.as_a_mut() {
                        a.0 = self.nat_router.route(a.0);
                    }
                }
            }
        }

        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);

        let response = builder.build(header, answers.iter(), &[], &[], &[]);

        Ok(responder.send_response(response).await?)
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> ResponseInfo {
        self.do_handle_request(request, response)
            .await
            .unwrap_or_else(|err| {
                error!("error in request handler: {err}");
                let mut header = Header::new();
                header.set_response_code(ResponseCode::ServFail);
                header.into()
            })
    }
}
