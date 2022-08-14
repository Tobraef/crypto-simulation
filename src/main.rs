mod AI;
mod domain;
mod web;

use std::{
    env::args,
    net::{Ipv4Addr, SocketAddrV4},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let port = args().nth(1).unwrap().parse().unwrap();
    AI::start(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).into()).await
}
