use std::str::FromStr;

use http::Uri;
use tower::ServiceBuilder;
use xscan_h2c::{
    protowire::{Ping, Pong, ping_pong_service_client::PingPongServiceClient},
    tonic::codec::CompressionEncoding::Gzip,
    web::GrpcWebClientLayer,
};

use crate::{error::Result, fetch::Fetch};

pub async fn pingpong(ping: Ping) -> Result<Pong> {
    let uri = Uri::from_str("http://127.0.0.1:3000")?;
    let service = ServiceBuilder::new().layer(GrpcWebClientLayer::new()).service(Fetch::new());

    let mut client = PingPongServiceClient::with_origin(service, uri);
    client = client.accept_compressed(Gzip).send_compressed(Gzip);

    let response = client.pingpong(ping).await?;
    Ok(response.into_inner())
}
