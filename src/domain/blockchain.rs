use std::ops::{Add, AddAssign, Sub};

use serde::{Deserialize, Serialize};

use super::{network::NodeId, transaction::ProvenTransaction};

pub const MAX_TRANSACTION_COUNT: usize = 10;

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

#[derive(Serialize, Deserialize)]
pub struct BlockHash(pub Vec<u8>);

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockIndex(pub usize);

pub struct Nonce(pub u8);

#[derive(Serialize, Deserialize)]
pub struct BlockHeader {
    pub index: BlockIndex,
    pub prev_hash: BlockHash,
    pub hash: BlockHash,
    pub timestamp: usize,
    pub difficulty: u8,
}

#[derive(Serialize, Deserialize)]
pub struct BlocksTransactions(pub Vec<ProvenTransaction>);

pub struct Block {
    pub header: BlockHeader,
    pub mined_by: NodeId,
    pub transactions: BlocksTransactions,
    pub nonce: usize,
}

pub struct Blockchain(pub Vec<Block>);
