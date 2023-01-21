use log::info;
use ring::ring_client::RingClient;
use ring::ring_server::{Ring, RingServer};
use ring::{JoinRequest, JoinResponse, SetHostRequest, SetHostResponse};
use tonic::client;
use tonic::{transport::Server, Request, Response, Status};

pub mod ring {
    tonic::include_proto!("ring");
}

#[derive(Debug, Default)]
pub struct MyRing {}

#[tonic::async_trait]
impl Ring for MyRing {
    async fn set_next(
        &self,
        request: Request<SetHostRequest>,
    ) -> Result<Response<SetHostResponse>, Status> {
        println!("Got Request: {:?}", request);

        let reply = SetHostResponse {};

        Ok(Response::new(reply))
    }

    async fn set_prev(
        &self,
        request: Request<SetHostRequest>,
    ) -> Result<Response<SetHostResponse>, Status> {
        let reply = SetHostResponse {};

        Ok(Response::new(reply))
    }

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<JoinResponse>, Status> {
        println!("Got Request: {:?}", request);

        let reply = JoinResponse {};

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = "[::1]:50051".parse().unwrap();
    let manager = MyRing::default();

    info!("server started");

    Server::builder()
        .add_service(RingServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}
