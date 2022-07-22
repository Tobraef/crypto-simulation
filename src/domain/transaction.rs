use super::{
    blockchain::NoCoin,
    network::{Network, Node, NodeId, User},
    rsa_verification::{encode_message, verify_message, RSAEncodedMsg},
    serialization::serialize,
    wallet::calculate_wallet,
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub from: Option<NodeId>,
    pub to: NodeId,
    pub fee: NoCoin,
    pub ammount: NoCoin,
}

#[derive(Serialize, Deserialize)]
pub struct AffordableTransaction(pub Transaction);

#[derive(Serialize, Deserialize)]
pub struct ProvenTransaction {
    pub transaction: AffordableTransaction,
    pub proof: RSAEncodedMsg,
}

impl Transaction {
    pub fn new(from: Option<NodeId>, to: NodeId, fee: NoCoin, ammount: NoCoin) -> Self {
        Self {
            from,
            to,
            fee,
            ammount,
        }
    }
}

pub fn create_transaction(
    network: &Network,
    recipient: &NodeId,
    ammount: NoCoin,
    fee: NoCoin,
) -> Result<ProvenTransaction> {
    let transaction = Transaction::new(
        Some(network.user.node.id.clone()),
        recipient.clone(),
        fee,
        ammount,
    );
    let affordable = map_to_affordable(&network, transaction)?;
    approve(affordable, &network.user)
}

fn approve(transaction: AffordableTransaction, user: &User) -> Result<ProvenTransaction> {
    let serialized = serialize(&transaction.0)?;
    let proof = encode_message(&serialized, &user.priv_key)?;
    Ok(ProvenTransaction { proof, transaction })
}

pub fn verify_transaction(
    network: &Network,
    transaction: Transaction,
    proof: Vec<u8>,
) -> Result<ProvenTransaction> {
    let transaction = map_to_affordable(network, transaction)?;
    prove_transaction(network, transaction, proof)
}

fn find_sender<'a>(network: &'a Network, id: &'a NodeId) -> Result<&'a Node> {
    network
        .nodes
        .iter()
        .find(|n| n.id == *id)
        .ok_or(anyhow!(format!("No node with id: {:?}", id)))
}

fn prove_transaction(
    network: &Network,
    transaction: AffordableTransaction,
    proof: Vec<u8>,
) -> Result<ProvenTransaction> {
    let sender = find_sender(network, transaction.0.from.as_ref().unwrap())?;
    let serialized = serialize(&transaction.0)?;
    let proof = verify_message(&serialized, proof, &sender.pub_key)?;
    Ok(ProvenTransaction { proof, transaction })
}

fn map_to_affordable(network: &Network, transaction: Transaction) -> Result<AffordableTransaction> {
    if let Some(sender) = transaction.from.as_ref() {
        let sender = find_sender(network, sender)?;
        let cash = network
            .cache
            .wallet
            .get(&sender.id)
            .map(|c| c.clone())
            .unwrap_or_else(|| calculate_wallet(&sender.id, &network.blockchain));
        if cash <= transaction.ammount + transaction.fee {
            Err(anyhow!(
                "Sender doesn't have enough coins to complete transaction.".to_owned()
            ))
        } else {
            Ok(AffordableTransaction(transaction))
        }
    } else {
        // mining reward, assumes it is affordable
        Ok(AffordableTransaction(transaction))
    }
}
