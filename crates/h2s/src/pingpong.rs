use xscan_h2c::{
    protowire::{
        Ping, Pong,
        ping_pong_service_server::{PingPongService, PingPongServiceServer},
    },
    tonic::{Request, Response, Status, async_trait},
};

#[derive(Debug, Default)]
pub struct PingPongServiceImpl {}

#[async_trait]
impl PingPongService for PingPongServiceImpl {
    async fn pingpong(&self, _: Request<Ping>) -> Result<Response<Pong>, Status> {
        Ok(Response::new(Pong::default()))
    }
}

pub fn service() -> PingPongServiceServer<PingPongServiceImpl> {
    PingPongServiceServer::new(PingPongServiceImpl::default())
}
