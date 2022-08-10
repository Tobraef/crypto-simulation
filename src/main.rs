mod AI;
mod domain;
mod web;

use std::net::{SocketAddrV4, Ipv4Addr};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    AI::start(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8100).into()).await
}
