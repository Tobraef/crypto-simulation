use std::{collections::HashMap, net::SocketAddrV4};

use serde::{Deserialize, Serialize};

use super::{
    blockchain::{Blockchain, NoCoin},
    rsa_verification::{PrivKey, PubKey},
    transaction::ProvenTransaction,
};

#[derive(Hash, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(usize);

impl NodeId {
    pub const MAX: NodeId = NodeId(usize::MAX);
}

pub struct Node {
    pub id: NodeId,
    pub addr: SocketAddrV4,
    pub pub_key: PubKey,
}

pub struct Network {
    pub user: User,
    pub nodes: Vec<Node>,
    pub blockchain: Blockchain,
    pub transactions_poll: Vec<ProvenTransaction>,
    pub cache: Cache,
}

pub struct Cache {
    pub wallet: HashMap<NodeId, NoCoin>,
}

pub struct User {
    pub node: Node,
    pub priv_key: PrivKey,
}
