use std::iter::once;

use futures::stream::FuturesUnordered;

use anyhow::bail;
use sha2::{Sha256, Digest};

use super::{blockchain::{Block, BlockHeader, Nonce, MAX_TRANSACTION_COUNT}, transaction::ProvenTransaction, network::NodeId, serialization::serialize};

pub async fn try_mine_any(difficulty: u8, miner: &NodeId, header: &BlockHeader, transactions: &Vec<ProvenTransaction>) -> Result<Nonce> {
    let mut split_pull = Vec::with_capacity(transactions.len() / MAX_TRANSACTION_COUNT);
    for i in (0..transactions.len()).step_by(10) {
        split_pull.push(&transactions[i..(i+MAX_TRANSACTION_COUNT).min(transactions.len())]);
    }
    let tasks: FuturesUnordered<_> = split_pull
        .into_iter()
        .map(|ts| mine(miner, header, ts, difficulty))
        .collect();
    tasks
        .into_iter()
        .filter_map(|x| x.await.ok())
        .next()
        .ok_or(anyhow!("Didn't find any block with given transactions."))
}

async fn mine(miner: &NodeId, header: &BlockHeader, transactions: &[ProvenTransaction], difficulty: u8) -> Result<Nonce> {
    let mut bytes = serialize(miner)?;
    bytes.extend_from_slice(&serialize(header))?;
    bytes.extend_from_slice(&serialize(transactions))?;
    bytes.extend_from_slice(&serialize(&difficulty))?;
    let mut sha256 = Sha256::new();
    tokio::task::spawn(move || {
        for nonce in 0..usize::MAX {
            sha256.chain_update(once(&bytes).chain(once(nonce.to_ne_bytes())));
            let result = sha256.finalize();
            if result[0..difficulty as usize].iter().all(|&b| b == b'0') {
                return Ok(nonce)
            }
        }
        bail!("No nonce found.")
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_difficulty() {
        
    }
}