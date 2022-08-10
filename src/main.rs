mod AI;
mod domain;
mod web;

use std::{net::{SocketAddrV4, Ipv4Addr}, env::args};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let port = args()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    AI::start(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).into()).await
}
