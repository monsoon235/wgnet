use tonic;
use tokio;
use tonic::{transport, Request, Response, Status};
use crate::config::server::ServerConfig;
use crate::api::proto;
use crate::api::proto::{GetPeersReply, GetPeersRequest, PingRequest, PingResponse, PostEndpointReply, PostEndpointRequest, RedeemInviteReply, RedeemInviteRequest};
use crate::wg::Interface;

pub struct Server {
    config: ServerConfig,
    iface: Interface,
}

#[derive(Debug, Default)]
struct RpcServer {}

#[tonic::async_trait]
impl proto::rpc_server::Rpc for RpcServer {
    async fn ping(&self, req: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        log::info!("Got ping msg from {}", req.into_inner().msg);
        let resp = PingResponse {
            msg: format!("Hi {}, I'm server", req.remote_addr().unwrap_or("0.0.0.0:0".parse().unwrap())),
        };
        Ok(Response::new(resp))
    }

    async fn redeem_invite(&self, req: Request<RedeemInviteRequest>) -> Result<Response<RedeemInviteReply>, Status> {
        todo!()
    }

    async fn post_endpoint(&self, req: Request<PostEndpointRequest>) -> Result<Response<PostEndpointReply>, Status> {
        todo!()
    }

    async fn get_peers(&self, req: Request<GetPeersRequest>) -> Result<Response<GetPeersReply>, Status> {
        todo!()
    }
}

impl Server {
    pub async fn run(&mut self) {
        transport::Server::builder()
            .add_service(RpcServer {})
            .serve(self.config.listen).await.unwrap();
    }
}


