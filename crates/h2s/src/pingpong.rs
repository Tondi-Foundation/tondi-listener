use tondi_scan_h2c::{
    protowire::{
        ping_pong_service_server::{PingPongService, PingPongServiceServer},
        Ping, Pong,
    },
    tonic::{Request, Response, Status},
};
use nill::{Nil, nil};

pub fn service() -> PingPongServiceServer<PingpongService> {
    PingPongServiceServer::new(PingpongService)
}

#[derive(Debug)]
pub struct PingpongService;

#[tonic::async_trait]
impl PingPongService for PingpongService {
    async fn pingpong(&self, request: Request<Ping>) -> Result<Response<Pong>, Status> {
        let ping = request.into_inner();
        let pong = Pong {
            id: format!("Pong: {}", ping.id),
        };
        Ok(Response::new(pong))
    }
}
