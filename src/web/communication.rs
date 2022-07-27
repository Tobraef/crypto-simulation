use std::net::SocketAddr;

use anyhow::{anyhow, bail, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use log::info;

use crate::domain::{Blockchain, Node, PubKey, User};

use super::server::ROUTES;

mod toolkit {
    use anyhow::Result;
    use log::info;
    use serde::Deserialize;
    use std::net::SocketAddr;

    async fn get_req(url: String) -> Result<String> {
        let response = reqwest::get(url).await?.text().await?;
        Ok(response)
    }

    fn map_resp<'a, T>(response: String) -> Result<T>
    where
        T: Deserialize<'a>,
    {
        let mapped = serde_json::from_str(&response)?;
        Ok(mapped)
    }

    fn url_for(addr: &SocketAddr, endpoint: &'static str) -> String {
        format!("http://{}/{}", addr.to_string(), endpoint)
    }

    pub async fn get_data<'a, T>(addr: &SocketAddr, endpoint: &'static str) -> Result<T>
    where
        T: Deserialize<'a>,
    {
        info!("Reaching endpoint: {}", endpoint);
        let url = url_for(addr, endpoint);
        let response = get_req(url).await?;
        info!("Received response: {}", response);
        let mapped = map_resp(response)?;
        Ok(mapped)
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
    client: &reqwest::Client,
    node: &Node,
    all_nodes: &[Node],
) -> Result<()> {
    let node_json = serde_json::to_vec(node)?;
    let mut tasks: FuturesUnordered<_> = all_nodes
        .iter()
        .filter(|n| n.id != node.id)
        .map(|n| {
            client
                .post(url_for(&node.addr, ROUTES.acknowledge_new_node))
                .body(node_json.clone())
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

pub async fn register_node(user: &User) -> Result<Vec<Node>> {
    let register_address = address_of_previous_node(&user.node.addr);
    toolkit::get_data(&register_address, ROUTES.register).await
}

pub async fn get_chain(node: &Node) -> Result<Blockchain> {
    toolkit::get_data(&node.addr, ROUTES.get_chain).await
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
