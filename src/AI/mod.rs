use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Result};

use log::info;
use tokio::{sync::Mutex, try_join};

use crate::{
    domain::{try_adopt_network, try_start_new_network, Network, try_adopt_pending_transactions, generate_key, try_mine_any_async, try_add_block, create_mined_block, create_mining_reward, ProvenTransaction, NodeId},
    web::{get_chain, register_node, run, get_pending_transactions, send_new_block},
};

async fn initialize_network(client: reqwest::Client, addr: SocketAddr) -> Result<Network> {
    let (private, public) = generate_key()?;
    match register_node(client, &addr, &public).await {
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
            let mut network = try_adopt_network(addr, private, public, nodes, blockchain)?;
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
            try_start_new_network(addr, private, public)
        }
    }
}

async fn mining_neccesities(network: Arc<Mutex<Network>>) -> (Vec<ProvenTransaction>, NodeId) {
    let network = network.lock().await;
    (network.transactions_poll.clone(), network.user.node.id.clone())
}


async fn mine_from_time_to_time(client: reqwest::Client, network: Arc<Mutex<Network>>) -> Result<()> {
    tokio::task::spawn(async move {
        loop {
            let (polled_transactions, user_id) = mining_neccesities(network.clone()).await;
            let reward = create_mining_reward(user_id);
            //mining should be interrupted, if some other node mines a block
            let mining_result = async {
                try_mine_any_async(5, &polled_transactions, &reward, user_id).await
            }.await;
            match mining_result {
                Ok((hash, nonce, transactions)) => {
                    info!("Successfully mined block, nonce: {:?}", nonce);
                    let mut network = network.lock().await;
                    let last_block = network.blockchain.last_block();
                    let mined_block = create_mined_block(last_block, hash, nonce, &transactions, user_id);
                    match try_add_block(&mut network, mined_block) {
                        Ok(()) => {
                            let added_block = network.blockchain.last_block().clone();
                            let other_nodes: Vec<_> = network.other_nodes().cloned().collect();
                            drop(network);
                            send_new_block(client.clone(), other_nodes, &added_block).await;
                        },
                        Err(e) => info!("Couldn't add mined block, reason: {}", e),
                    }
                },
                Err(e) => info!("Error from mining block: {:?}", e),
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    })
    .await?;
    Ok(())
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    let client = reqwest::Client::new();
    let network  = Arc::new(Mutex::new(initialize_network(client.clone(), addr).await?));

    let run_server = run(addr, network.clone());
    let mining = mine_from_time_to_time(client.clone(), network.clone());

    try_join!(run_server, mining)?;
    Ok(())    
}
