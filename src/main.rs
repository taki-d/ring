use tonic::{transport::Server, Request, Response, Status};
use ring::{ JoinRequest, JoinResponse, SetHostRequest, SetHostResponse };
use ring::ring_server::{Ring, RingServer};

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

        let reply = SetHostResponse{};

        Ok(Response::new(reply))
    }

    async fn set_prev(
        &self,
        request: Request<SetHostRequest>,
    ) -> Result<Response<SetHostResponse>, Status> {

        let reply = SetHostResponse{};

        Ok(Response::new(reply))
    }

    async fn join(
        &self,
        request: Request<JoinRequest>,
    ) -> Result<Response<JoinResponse>, Status> {

        let reply = JoinResponse{};

        Ok(Response::new(reply))
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("Hello, world!");

    let addr = "[::1]:50051".parse().unwrap();
    let manager = MyRing::default();

    Server::builder()
        .add_service(RingServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}
