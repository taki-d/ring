use std::env;
use std::net::SocketAddr;
use std::sync::RwLock;

use clap::Parser;
use log::info;
use ring::ring_client::RingClient;
use ring::ring_server::{Ring, RingServer};
use ring::{JoinRequest, JoinResponse, SetHostRequest, SetHostResponse};
use tonic::client;
use tonic::{transport::Server, Request, Response, Status};

pub mod ring {
    tonic::include_proto!("ring");
}

#[derive(Debug)]
pub struct MyRing {
    addr: RwLock<SocketAddr>,
    next_addr: RwLock<SocketAddr>,
    prev_addr: RwLock<SocketAddr>,
}

impl MyRing {
    pub fn new(address: SocketAddr) -> MyRing {
        MyRing {
            addr: RwLock::new(address),
            next_addr: RwLock::new(address),
            prev_addr: RwLock::new(address),
        }
    }
}

#[tonic::async_trait]
impl Ring for MyRing {
    async fn set_next(
        &self,
        request: Request<SetHostRequest>,
    ) -> Result<Response<SetHostResponse>, Status> {
        info!("set_next rpc called: {:?}", request);

        let r = request.into_inner();
        *self.next_addr.write().unwrap() = r.host.parse().unwrap();

        let reply = SetHostResponse {};
        Ok(Response::new(reply))
    }

    async fn set_prev(
        &self,
        request: Request<SetHostRequest>,
    ) -> Result<Response<SetHostResponse>, Status> {
        info!("set_prev rpc called: {:?}", request);
        let reply = SetHostResponse {};

        let r = request.into_inner();
        *self.prev_addr.write().unwrap() = r.host.parse().unwrap();

        Ok(Response::new(reply))
    }

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<JoinResponse>, Status> {
        info!("join rpc called: {:?}", request);

        let reply = JoinResponse {
            host: self.addr.read().unwrap().to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[derive(Parser, Debug)]
struct Args {
    addr: String,
    next_addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();

    let addr = args.addr.parse().unwrap();
    let manager = MyRing::new(addr);

    info!("server started");
    info!("params: {:?}", args);

    Server::builder()
        .add_service(RingServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}
