use std::{collections::HashMap, net::SocketAddr};

use anyhow::{anyhow, bail, Result};

use serde::{Deserialize, Serialize};

use super::{
    blockchain::{genesis_block, verify_blockchain, Blockchain, NoCoin, Nonce, BlocksTransactions, BlockHeader},
    mining::{prove_mined_block, BlockHash},
    rsa_verification::{generate_key, PrivKey, PubKey},
    transaction::{verify_transaction, ProvenTransaction},
    Block, Transaction,
};

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(pub usize);

impl NodeId {
    pub const MAX: NodeId = NodeId(usize::MAX);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: NodeId,
    pub addr: SocketAddr,
    pub pub_key: PubKey,
}

pub struct Network {
    pub user: User,
    pub nodes: Vec<Node>,
    pub blockchain: Blockchain,
    pub transactions_poll: Vec<ProvenTransaction>,
    pub cache: Cache,
    _void: (),
}

impl Network {
    pub fn other_nodes(&self) -> impl Iterator<Item=&Node> {
        self.nodes.iter().filter(|n| n.id != self.user.node.id)
    }
}

pub struct Cache {
    pub wallet: HashMap<NodeId, NoCoin>,
}

pub struct User {
    pub node: Node,
    pub priv_key: PrivKey,
}

pub fn try_create_node(network: &mut Network, addr: SocketAddr, key: PubKey) -> Result<&Node> {
    let id = NodeId(addr.port().into());
    if network.nodes.iter().any(|n| n.id == id) {
        Err(anyhow::anyhow!(
            "There already exists node with port {}, try again later.",
            id.0
        ))
    } else {
        let node = Node {
            id,
            addr,
            pub_key: key,
        };
        network.nodes.push(node);
        Ok(&network.nodes.last().unwrap())
    }
}

pub fn acknowledge_node(network: &mut Network, node: Node) -> Result<()> {
    if network.nodes.iter().any(|n| n.id == node.id) {
        bail!("Node {:?} is already in the network", node)
    } else {
        network.nodes.push(node);
        Ok(())
    }
}

pub fn try_add_transaction(
    network: &mut Network,
    transaction: Transaction,
    proof: Vec<u8>,
) -> Result<()> {
    let transaction = verify_transaction(network, transaction, proof)?;
    network.transactions_poll.push(transaction);
    Ok(())
}

fn remove_transactions_from_poll(poll: &mut Vec<ProvenTransaction>, transactions: &[ProvenTransaction]) -> Result<()> {
    let removable_from_poll = transactions
        .iter()
        .filter(|t| poll.iter().any(|x| x.transaction.0 == t.transaction.0))
        .collect::<Vec<_>>();
    if removable_from_poll.len() < transactions.len() - 1 {
        bail!("Transactions in the block (except reward) are not in the poll. Rejecting block.");
    } else {
        poll.retain(|t| transactions.iter().all(|x| x.transaction.0 != t.transaction.0));
        Ok(())
    }
}

pub fn try_add_block(network: &mut Network, block: Block) -> Result<()> {
    prove_mined_block(&block.transactions.0, block.header.difficulty, block.nonce)?;
    remove_transactions_from_poll(&mut network.transactions_poll, &block.transactions.0)?;
    network.blockchain.0.push(block);
    Ok(())
}

fn new_user(addr: SocketAddr, priv_key: PrivKey, pub_key: PubKey) -> Result<User> {
    Ok(User {
        node: Node {
            id: NodeId(addr.port() as usize),
            addr,
            pub_key,
        },
        priv_key,
    })
}

pub fn try_start_new_network(
    addr: SocketAddr,
    priv_key: PrivKey,
    pub_key: PubKey,
) -> Result<Network> {
    let user = new_user(addr, priv_key, pub_key)?; 
    let node = user.node.clone();
    Ok(Network {
        user: user,
        nodes: vec![node],
        blockchain: Blockchain(vec![genesis_block()]),
        transactions_poll: vec![],
        cache: Cache {
            wallet: HashMap::new(),
        },
        _void: (),
    })
}

pub fn try_adopt_network(
    addr: SocketAddr,
    priv_key: PrivKey,
    pub_key: PubKey,
    nodes: Vec<Node>,
    chain: Blockchain,
) -> Result<Network> {
    let chain = verify_blockchain(chain)?;
    Ok(Network {
        user: new_user(addr, priv_key, pub_key)?,
        nodes,
        blockchain: chain,
        transactions_poll: vec![],
        cache: Cache {
            wallet: HashMap::new(),
        },
        _void: (),
    })
}

pub fn try_adopt_pending_transactions(
    network: &mut Network,
    transactions: Vec<(Transaction, Vec<u8>)>,
) -> Result<()> {
    let verification_results: Vec<_> = transactions
        .into_iter()
        .map(|(transaction, proof)| verify_transaction(network, transaction, proof))
        .collect();
    if verification_results.iter().any(|r| r.is_err()) {
        let error_text = verification_results
            .into_iter()
            .filter_map(|v| v.err().map(|e| e.to_string()))
            .fold(String::default(), |e1, e2| format!("{e1}\n{e2}"));
        bail!(error_text)
    } else {
        let verified_transactions = verification_results
            .into_iter()
            .filter_map(|f| f.ok())
            .collect();
        network.transactions_poll = verified_transactions;
        Ok(())
    }
}

pub fn create_mined_block(prev_block: &Block, hash: BlockHash, nonce: Nonce, transactions: &[&ProvenTransaction], miner: NodeId) -> Block {
    Block {
        header: BlockHeader::new(prev_block, hash),
        mined_by: miner,
        transactions: BlocksTransactions(transactions.iter().map(|&x| x.clone()).collect()),
        nonce,
    }
}
