use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Result};

use log::info;
use tokio::{sync::Mutex, try_join};

use crate::{
    domain::{try_adopt_network, try_start_new_network, Network, User, try_adopt_pending_transactions},
    web::{get_chain, register_node, run, get_pending_transactions},
};

async fn initialize_network(addr: SocketAddr) -> Result<Network> {
    match register_node(&addr).await {
        Ok(nodes) => {
            info!(
                "Node succesfully registered. Received {} nodes.",
                nodes.len()
            );
            let node_to_talk = nodes
                .first()
                .ok_or(anyhow!("Received empty nodes from register"))?;
            let blockchain = get_chain(node_to_talk).await?;
            info!("Received blockchain: {:?}", blockchain);
            let mut network = try_adopt_network(addr, nodes, blockchain)?;
            let transactions = get_pending_transactions(network.nodes.first().unwrap()).await?;
            info!("Received pending transactions: {:?}", transactions);
            try_adopt_pending_transactions(&mut network, transactions)?;
            Ok(network)
        }
        Err(e) => {
            info!(
                "Couldn't register node, starting own network. Error from registering: {}",
                e
            );
            try_start_new_network(addr)
        }
    }
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    let network  = Arc::new(Mutex::new(initialize_network(addr).await?));

    let run_server = run(addr, network.clone());

    run_server.await
}
