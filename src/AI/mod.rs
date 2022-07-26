use std::{net::SocketAddr, sync::Arc};

use anyhow::{Result, anyhow};

use tokio::{sync::Mutex, try_join};

use crate::{web::{run, register_node, get_chain}, domain::{try_start_new_network, Network, try_adopt_network, try_adopt_blockchain}};

async fn initialize_network(network: Arc<Mutex<Network>>) -> Result<()> {
    let mut network = network.lock().await;
    match register_node(&network.user).await {
        Ok(nodes) => {
            let node_to_talk = nodes.first().ok_or(anyhow!("Received empty nodes from register"))?;
            let blockchain = get_chain(node_to_talk).await?;
            try_adopt_blockchain(&mut network, blockchain)?;
            network.nodes = nodes;
        },
        Err(_) => todo!(),
    }
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    let network = try_start_new_network(addr.clone())?;
    let network = Arc::new(Mutex::new(network));

    let run_server = run(addr, network.clone());
    let initialize_network = initialize_network(network.clone());

    try_join!(run_server, initialize_network)
        .map(|_| ())
}
