use std::iter::once;

use futures::{stream::FuturesUnordered, StreamExt};

use anyhow::{bail, Result};
use log::debug;
use sha2::{Digest, Sha256};

use super::{
    blockchain::{BlockHeader, BlocksTransactions, Nonce, MAX_TRANSACTION_COUNT},
    network::NodeId,
    serialization::serialize,
    transaction::ProvenTransaction,
};

pub async fn try_mine_any(difficulty: u8, transactions: &Vec<ProvenTransaction>) -> Result<Nonce> {
    let mut split_pull: Vec<Vec<&ProvenTransaction>> =
        Vec::with_capacity(transactions.len() / MAX_TRANSACTION_COUNT);
    for i in (0..transactions.len()).step_by(10) {
        split_pull.push(
            transactions
                .iter()
                .skip(i)
                .take(MAX_TRANSACTION_COUNT)
                .collect(),
        );
    }
    let mut tasks: FuturesUnordered<_> = split_pull
        .into_iter()
        .map(|ts| mine(ts, difficulty))
        .collect();
    if let Some(Ok(nonce)) = tasks.next().await {
        Ok(nonce)
    } else {
        bail!("No nonce could create block with given set of transactions")
    }
}

async fn mine(transactions: Vec<&ProvenTransaction>, difficulty: u8) -> Result<Nonce> {
    let block_data = serialize(&transactions)?;
    tokio::task::spawn(async move {
        (0..u32::MAX)
            .map(|n| (n, hash_block(&block_data, n)))
            .find(|(_n, hash)| hash[..difficulty as usize].chars().all(|b| b == '0'))
            .map(|(n, _hash)| Nonce(n as u8))
            .ok_or(anyhow::anyhow!("No nonce found."))
    })
    .await?
}

fn hash_block(block_data: &[u8], nonce: u32) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(block_data);
    sha256.update(nonce.to_ne_bytes());
    format!("{:x}", sha256.finalize())
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    use crate::domain::{
        blockchain::NoCoin,
        rsa_verification::{encode_message, generate_key, RSAEncodedMsg},
        transaction::{AffordableTransaction, Transaction},
    };

    use super::*;

    fn some_transactions() -> Vec<ProvenTransaction> {
        let mut seed = [0; 32];
        let key = generate_key().unwrap();

        seed[0] = 1;
        seed[1] = 0xA;
        seed[2] = 0xC;
        let mut rng = rand::prelude::StdRng::from_seed(seed);
        (0..rng.gen_range(1..=10))
            .map(|_| ProvenTransaction {
                proof: encode_message(&vec![rng.gen(), rng.gen()], &key).unwrap(),
                transaction: AffordableTransaction(Transaction::new(
                    Some(NodeId(rng.gen())),
                    NodeId(rng.gen()),
                    NoCoin(rng.gen()),
                    NoCoin(rng.gen()),
                )),
            })
            .collect()
    }

    #[tokio::test]
    async fn impossible_difficulty() {
        let transactions = BlocksTransactions(some_transactions());
        let mine_result = mine(transactions.0.iter().collect(), 3).await.unwrap();

        //assert!(mine_result.is_err())
    }
}
