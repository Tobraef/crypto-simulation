use std::ops::{Add, AddAssign, Sub};

use serde::{Deserialize, Serialize};

use super::{
    mining::{prove_mined_block, try_mine_any, BlockHash},
    network::NodeId,
    transaction::ProvenTransaction,
};

use anyhow::{anyhow, bail, Result};

pub const MAX_TRANSACTION_COUNT: usize = 10;
const GENESIS_DIFFICULTY: u8 = 3;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct NoCoin(pub f32);

impl Add for NoCoin {
    type Output = NoCoin;
    fn add(self, rhs: Self) -> Self::Output {
        NoCoin(self.0 + rhs.0)
    }
}

impl Sub for NoCoin {
    type Output = NoCoin;
    fn sub(self, rhs: Self) -> Self::Output {
        NoCoin(self.0 - rhs.0)
    }
}

impl AddAssign for NoCoin {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockIndex(pub usize);

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Nonce(pub u32);

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub index: BlockIndex,
    pub prev_hash: BlockHash,
    pub hash: BlockHash,
    pub timestamp: usize,
    pub difficulty: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlocksTransactions(pub Vec<ProvenTransaction>);

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub mined_by: NodeId,
    pub transactions: BlocksTransactions,
    pub nonce: Nonce,
}

//Should be VerifiedBlockchain some day.
//Currently get_chain is received with 'validated' transactions which doesn't have to be true
#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain(pub Vec<Block>);

pub fn genesis_block() -> Block {
    let (hash, nonce) = try_mine_any(GENESIS_DIFFICULTY, &vec![])
        .expect("Couldn't create genesis block. Aborting.");
    Block {
        header: BlockHeader {
            index: BlockIndex(0),
            prev_hash: BlockHash::default(),
            hash,
            timestamp: std::time::Instant::now().elapsed().as_secs() as usize,
            difficulty: GENESIS_DIFFICULTY,
        },
        mined_by: NodeId(0),
        transactions: BlocksTransactions(vec![]),
        nonce,
    }
}

fn has_valid_genesis_block(blockchain: &Blockchain) -> Result<()> {
    let genesis_block = blockchain
        .0
        .first()
        .ok_or(anyhow!("Didn't find genesis block. Blockchain is empty."))?;
    if !genesis_block.transactions.0.is_empty() {
        bail!("Genesis block should have no transactions.")
    }
    if !genesis_block.header.prev_hash.0.bytes().any(|b| b != b'0') {
        bail!(
            "Genesis block should have previous has as only zeroes. But is {:?}",
            genesis_block.header.prev_hash.0
        )
    }
    if genesis_block.header.difficulty != GENESIS_DIFFICULTY {
        bail!(
            "Genesis block should have difficulty of 5. Was {}",
            genesis_block.header.difficulty
        )
    }
    let _ = prove_mined_block(
        &genesis_block.transactions.0,
        genesis_block.header.difficulty,
        genesis_block.nonce,
    )?;
    Ok(())
}

pub fn verify_blockchain(blockchain: Blockchain) -> Result<Blockchain> {
    let _ = has_valid_genesis_block(&blockchain)?;
    let chain = &blockchain.0;
    for i in 1..blockchain.0.len() {
        let previous_hash = &chain[i - 1].header.hash;
        let block_to_verify = &chain[i];
        if *previous_hash != block_to_verify.header.prev_hash {
            bail!(
                "Previous block hash doesn't match current hash. Invalid block: {:?}",
                block_to_verify
            )
        }
        let _ = prove_mined_block(
            &block_to_verify.transactions.0,
            block_to_verify.header.difficulty,
            block_to_verify.nonce,
        )
        .map_err(|_| {
            anyhow!(
                "Block is fake. Hash doesn't match its nonce. Invalid block: {:?}",
                block_to_verify
            )
        })?;
    }
    Ok(blockchain)
}
