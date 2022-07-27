use std::iter::once;

use futures::{stream::FuturesUnordered, StreamExt};

use anyhow::{bail, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    blockchain::{BlockHeader, BlocksTransactions, Nonce, MAX_TRANSACTION_COUNT},
    network::NodeId,
    serialization::serialize,
    transaction::ProvenTransaction,
};

const HASH_LEN: usize = 64;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct BlockHash(pub String);

impl Default for BlockHash {
    fn default() -> Self {
        Self("0".repeat(HASH_LEN))
    }
}

impl BlockHash {
    pub fn try_new(bytes: String) -> Result<Self> {
        if bytes.len() != HASH_LEN {
            bail!("Hash must be {} bits long", HASH_LEN)
        } else {
            Ok(Self(bytes))
        }
    }
}

pub async fn try_mine_any_async(
    difficulty: u8,
    transactions: &Vec<ProvenTransaction>,
) -> Result<(BlockHash, Nonce)> {
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
        .map(|ts| mine_async(ts, difficulty))
        .collect();
    if let Some(Ok(nonce)) = tasks.next().await {
        Ok(nonce)
    } else {
        bail!("No nonce could create block with given set of transactions")
    }
}

pub fn try_mine_any(
    difficulty: u8,
    transactions: &Vec<ProvenTransaction>,
) -> Result<(BlockHash, Nonce)> {
    todo!("Should be possible, when transactions are empty");
    for i in (0..transactions.len()).step_by(MAX_TRANSACTION_COUNT) {
        let transactions = &transactions[i..transactions.len().min(i + MAX_TRANSACTION_COUNT)];
        if let Ok(r) = mine(transactions, difficulty) {
            return Ok(r);
        }
    }
    bail!("No nonce found.")
}

async fn mine_async(
    transactions: Vec<&ProvenTransaction>,
    difficulty: u8,
) -> Result<(BlockHash, Nonce)> {
    let block_data = serialize(&transactions)?;
    tokio::task::spawn(async move {
        (0..u32::MAX)
            .map(|n| (Nonce(n), hash_block(&block_data, Nonce(n))))
            .find(|(_n, hash)| hash_matches(hash, difficulty))
            .map(|(n, hash)| (BlockHash(hash), n))
            .ok_or(anyhow::anyhow!("No nonce found."))
    })
    .await?
}

fn mine(transactions: &[ProvenTransaction], difficulty: u8) -> Result<(BlockHash, Nonce)> {
    let block_data = serialize(&transactions)?;
    (0..u32::MAX)
        .map(|n| (Nonce(n), hash_block(&block_data, Nonce(n))))
        .find(|(_n, hash)| hash_matches(hash, difficulty))
        .map(|(n, hash)| (BlockHash(hash), n))
        .ok_or(anyhow::anyhow!("No nonce found."))
}

fn hash_matches(hash: &str, difficulty: u8) -> bool {
    hash[..difficulty as usize].chars().all(|b| b == '0')
}

pub fn prove_mined_block(
    transactions: &Vec<ProvenTransaction>,
    difficulty: u8,
    nonce: Nonce,
) -> Result<()> {
    let block_data = serialize(transactions)?;
    let hash = hash_block(&block_data, nonce);
    if hash_matches(&hash, difficulty) {
        Ok(())
    } else {
        bail!("Block is not valid")
    }
}

fn hash_block(block_data: &[u8], nonce: Nonce) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(block_data);
    sha256.update(nonce.0.to_ne_bytes());
    format!("{:x}", sha256.finalize())
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    use crate::domain::{
        blockchain::NoCoin,
        rsa_verification::{encode_message, generate_key},
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
                proof: encode_message(&vec![rng.gen(), rng.gen()], &key.0).unwrap(),
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
    async fn possible_difficulty() {
        let transactions = BlocksTransactions(some_transactions());

        for dif in 1..4 {
            let (_, nonce) = mine_async(transactions.0.iter().collect(), dif as u8)
                .await
                .unwrap();

            let hash = hash_block(
                &serialize::<Vec<&ProvenTransaction>>(&transactions.0.iter().collect()).unwrap(),
                nonce,
            );

            assert_eq!(
                hash[..dif],
                "0".repeat(dif),
                "Not zeroized for difficulty: {}",
                dif
            );
        }
    }

    #[tokio::test]
    async fn hash_and_verify() {
        let transactions = some_transactions();
        let difficulty = 3;

        let (_, nonce) = mine_async(transactions.iter().collect(), difficulty)
            .await
            .unwrap();
        let proof = prove_mined_block(&transactions, difficulty, nonce);

        assert!(proof.is_ok());

        let invalid_proof = prove_mined_block(&transactions, difficulty, Nonce(nonce.0 + 1));

        assert!(invalid_proof.is_err());
    }
}
