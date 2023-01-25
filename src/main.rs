use std::env;
use std::fmt::format;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ptr::addr_of_mut;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

use clap::Parser;
use futures::executor::block_on;
use log::info;
use ring::ring_client::RingClient;
use ring::ring_server::RingServer;
use ring::{JoinRequest, JoinResponse, SetHostRequest, SetHostResponse};
use tonic::client;
use tonic::codegen::http::header::IntoHeaderName;
use tonic::transport::Channel;
use tonic::{transport::Server, Request, Response, Status};

pub mod ring {
    tonic::include_proto!("ring");
}

#[derive(Debug)]
pub struct Connection {
    next_addr: RwLock<SocketAddr>,
    prev_addr: RwLock<SocketAddr>,
    // next_con: Mutex<RingClient<Channel>>,
    // prev_con: Mutex<RingClient<Channel>>,
}

#[derive(Debug)]
pub struct Ring {
    addr: RwLock<SocketAddr>,
    connection: Arc<Connection>,
}

impl Ring {
    pub fn new(address: SocketAddr, connection: Arc<Connection>) -> Ring {
        Ring {
            addr: RwLock::new(address),
            connection: connection,
        }
    }

    pub async fn join(&self, next_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        *self.connection.next_addr.write().await = next_addr;
        let addr = format!("http://{}", &self.connection.next_addr.read().await);
        let mut next = RingClient::connect(addr).await.unwrap();

        let req = tonic::Request::new(JoinRequest {
            host: self.addr.read().await.to_string(),
        });
        let res = next.join(req).await?;

        info!("join rpc response: {:?}", res);

        *self.connection.prev_addr.write().await = res.into_inner().host.parse().unwrap();

        info!(
            "prev: {}, next {}",
            self.connection.prev_addr.read().await,
            self.connection.next_addr.read().await
        );

        Ok(())
    }
}

#[tonic::async_trait]
impl ring::ring_server::Ring for Ring {
    async fn set_next(
        &self,
        request: Request<SetHostRequest>,
    ) -> Result<Response<SetHostResponse>, Status> {
        info!("set_next rpc called: {:?}", request);

        let r = request.into_inner();
        *self.connection.next_addr.write().await = r.host.parse().unwrap();
        *self.connection.next_addr.write().await = r.host.parse().unwrap();

        let reply = SetHostResponse {};

        info!(
            "prev: {}, next {}",
            self.connection.prev_addr.read().await,
            self.connection.next_addr.read().await
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
        *self.connection.prev_addr.write().await = r.host.parse().unwrap();

        info!(
            "prev: {}, next {}",
            self.connection.prev_addr.read().await,
            self.connection.next_addr.read().await
        );

        Ok(Response::new(reply))
    }

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<JoinResponse>, Status> {
        info!("join rpc called: {:?}", request);
        let addr = format!("http://{}", &self.connection.prev_addr.read().await);

        let mut prev = RingClient::connect(addr).await.unwrap();
        let next_addr: SocketAddr = request.into_inner().host.clone().parse().unwrap();

        let req = tonic::Request::new(SetHostRequest {
            host: next_addr.to_string(),
        });

        let _ = prev.set_next(req).await;

        let reply = JoinResponse {
            host: self.connection.prev_addr.read().await.to_string(),
        };

        *self.connection.prev_addr.write().await = next_addr;

        info!(
            "prev: {}, next {}",
            self.connection.prev_addr.read().await,
            self.connection.next_addr.read().await
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

    let con = Arc::new(Connection {
        next_addr: RwLock::new(addr),
        prev_addr: RwLock::new(addr),
    });

    let manager = Ring::new(addr, con.clone());

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
