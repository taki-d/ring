use std::env;
use std::fmt::format;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::RwLock;

use clap::Parser;
use futures::executor::block_on;
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

    pub async fn join(&self, next_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        *self.next_addr.write().unwrap() = next_addr;
        let addr = format!("http://{}", &self.next_addr.read().unwrap());
        let mut next = RingClient::connect(addr).await.unwrap();

        let req = tonic::Request::new(JoinRequest {
            host: self.addr.read().unwrap().to_string(),
        });
        let res = next.join(req).await?;

        info!("join rpc response: {:?}", res);

        *self.prev_addr.write().unwrap() = res.into_inner().host.parse().unwrap();

        info!(
            "prev: {}, next {}",
            self.prev_addr.read().unwrap(),
            self.next_addr.read().unwrap()
        );

        Ok(())
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

        info!(
            "prev: {}, next {}",
            self.prev_addr.read().unwrap(),
            self.next_addr.read().unwrap()
        );

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

        info!(
            "prev: {}, next {}",
            self.prev_addr.read().unwrap(),
            self.next_addr.read().unwrap()
        );

        Ok(Response::new(reply))
    }

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<JoinResponse>, Status> {
        info!("join rpc called: {:?}", request);
        let addr = format!("http://{}", &self.prev_addr.read().unwrap());

        let mut prev = RingClient::connect(addr).await.unwrap();
        let next_addr: SocketAddr = request.into_inner().host.clone().parse().unwrap();

        let req = tonic::Request::new(SetHostRequest {
            host: next_addr.to_string(),
        });

        let _ = prev.set_next(req).await;

        let reply = JoinResponse {
            host: self.prev_addr.read().unwrap().to_string(),
        };

        *self.prev_addr.write().unwrap() = next_addr;

        info!(
            "prev: {}, next {}",
            self.prev_addr.read().unwrap(),
            self.next_addr.read().unwrap()
        );
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
    info!("params: {:?}", args);

    let addr = args.addr.parse().unwrap();
    let manager = MyRing::new(addr);

    match args.next_addr {
        Some(next_addr) => {
            manager.join(next_addr.parse().unwrap()).await?;
        }
        None => {
            info!("next_addres is not provided");
        }
    }
    info!("server started");

    Server::builder()
        .add_service(RingServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}
