use std::net::SocketAddr;

use anyhow::{anyhow, bail, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use log::info;

use crate::domain::{Blockchain, Node, PubKey, Transaction, User, Block};

use self::toolkit::url_for;

use super::server::ROUTES;

mod toolkit {
    use anyhow::Result;
    use log::info;
    use serde::de::DeserializeOwned;
    use std::{fmt::Debug, net::SocketAddr};

    async fn get_req<T>(url: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = reqwest::get(url).await?.json().await?;
        Ok(response)
    }

    pub(super) fn url_for(addr: &SocketAddr, endpoint: &'static str) -> String {
        format!("http://{}/{}", addr.to_string(), endpoint)
    }

    pub(super) async fn get_data<T>(addr: &SocketAddr, endpoint: &'static str) -> Result<T>
    where
        T: DeserializeOwned + Debug,
    {
        info!("Reaching endpoint: {}", endpoint);
        let url = url_for(addr, endpoint);
        let response = get_req(url).await?;
        info!("Received response: {:?}", response);
        Ok(response)
    }
}

pub async fn get_bitcoin_value_usd() -> Result<f32> {
    let response = reqwest::get("https://blockchain.info/ticker")
        .await?
        .text()
        .await?;
    let map: serde_json::Map<_, _> = serde_json::from_str(&response)?;
    let usd = map
        .get("USD")
        .ok_or(anyhow!("Didn't find USD node in response."))?;
    let last = usd.get("last").ok_or(anyhow!(
        "Didn't find 'last' in USD node. Node was: {:?}",
        usd
    ))?;
    match last {
        serde_json::Value::Number(n) => {
            let value = n
                .as_f64()
                .ok_or(anyhow!("Last number value is not a float."))?;
            Ok(value as f32)
        }
        _ => bail!("Last has invalid type. Expected number, was: {:?}", last),
    }
}

pub async fn send_acknowledge_new_node(
    user: &User,
    client: &reqwest::Client,
    node: &Node,
    all_nodes: &[Node],
) -> Result<()> {
    let mut tasks: FuturesUnordered<_> = all_nodes
        .iter()
        .filter(|n| n.id != node.id && n.id != user.node.id)
        .map(|n| {
            client
                .post(toolkit::url_for(&n.addr, ROUTES.acknowledge_new_node))
                .json(node)
                .send()
        })
        .collect();
    while let Some(Err(e)) = tasks.next().await {
        info!("Received error sending acknowledge to node: {}", e);
    }
    Ok(())
}

fn address_of_previous_node(addr: &SocketAddr) -> SocketAddr {
    let mut copy = addr.clone();
    copy.set_port(copy.port() - 1);
    copy
}

pub async fn register_node(client: reqwest::Client, addr: &SocketAddr, pub_key: &PubKey) -> Result<Vec<Node>> {
    let register_address = address_of_previous_node(addr);
    Ok(client
        .post(url_for(&register_address, ROUTES.register))
        .json(pub_key)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_chain(node: &Node) -> Result<Blockchain> {
    toolkit::get_data(&node.addr, ROUTES.get_chain).await
}

pub async fn get_pending_transactions(node: &Node) -> Result<Vec<(Transaction, Vec<u8>)>> {
    toolkit::get_data(&node.addr, ROUTES.get_pending_transactions).await
}

pub async fn send_new_block(client: reqwest::Client, recipients: Vec<Node>, block: &Block) 
{
    info!("Sending block to nodes {:?}", recipients.iter().map(|x| x.addr).collect::<Vec<_>>());
    let mut tasks: FuturesUnordered<_> = recipients.iter()
        .map(|r| url_for(&r.addr, ROUTES.new_block))
        .map(|url| client
            .post(url)
            .json(block)
            .send())
        .collect();
    while let Some(r) = tasks.next().await {
        if let Err(e) = r {
            info!("Received error sending new block {:?}", e);
        }
    }
    info!("Finished sending");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bitcoin_api() {
        let value = get_bitcoin_value_usd().await;

        assert!(value.is_ok(), "Failed with err {:?}", value);

        assert!(
            value.unwrap() > 1000.,
            "Bitcoin cheaper than 1k USD holy cow."
        ); //i guess it will always be more expensive than this
    }
}
