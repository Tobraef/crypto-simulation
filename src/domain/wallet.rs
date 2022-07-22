use std::collections::HashMap;

use super::{
    blockchain::{Block, Blockchain, NoCoin},
    network::NodeId,
    transaction::Transaction,
};

pub fn calculate_all_wallets(blockchain: &Blockchain) -> HashMap<NodeId, NoCoin> {
    let mut result = HashMap::new();
    for block in blockchain.0.iter() {
        *result.entry(block.mined_by.clone()).or_insert(NoCoin(0.)) += mining_fees_gain(block);
        for proven_transaction in block.transactions.0.iter() {
            let transaction = &proven_transaction.transaction.0;
            if let Some(from) = transaction.from.as_ref() {
                *result.entry(from.clone()).or_insert(NoCoin(0.)) +=
                    NoCoin(0.) - transaction.ammount - transaction.fee;
            }
            *result.entry(transaction.to.clone()).or_insert(NoCoin(0.)) += transaction.ammount;
        }
    }
    result
}

pub fn calculate_wallet(id: &NodeId, blockchain: &Blockchain) -> NoCoin {
    let from_transactions = transactions_gain_for(id, blockchain);
    let from_mining = mining_fees_gain_for(id, blockchain);
    from_transactions + from_mining
}

fn mining_fees_gain(block: &Block) -> NoCoin {
    NoCoin(
        block
            .transactions
            .0
            .iter()
            .map(|t| t.transaction.0.fee.0)
            .sum(),
    )
}

fn mining_fees_gain_for(id: &NodeId, blockchain: &Blockchain) -> NoCoin {
    let sum = blockchain
        .0
        .iter()
        .filter(|b| b.mined_by == *id)
        .map(|b| mining_fees_gain(b).0)
        .sum();
    NoCoin(sum)
}

fn transactions_gain_for(id: &NodeId, blockchain: &Blockchain) -> NoCoin {
    blockchain
        .0
        .iter()
        .flat_map(|c| c.transactions.0.iter())
        .fold(NoCoin(0.), |acc, t| {
            acc + balance_after_transaction(id, &t.transaction.0)
        })
}

fn balance_after_transaction(id: &NodeId, transaction: &Transaction) -> NoCoin {
    if transaction.to == *id {
        return transaction.ammount;
    } else {
        if let Some(s) = transaction.from.as_ref() {
            if s == id {
                return NoCoin(0.) - transaction.ammount - transaction.fee;
            }
        }
    }
    NoCoin(0.)
}

// #[cfg(test)]
// mod tests {
//     use crate::domain::Block;

//     use super::*;

//     #[test]
//     fn calculate_transaction_gain_no_fees_no_rewards() {
//         let target = NodeId(0);
//         let blockchain = Blockchain(vec![mock_block!(
//             2->0,f 0.,a 5.;
//             1->0,f 0.,a 6.;
//             0->1,f 0.,a 3.;
//             0->2,f 0.,a 2.| 0
//         )]);

//         assert_eq_coin(
//             calculate_wallet(&target, &blockchain),
//             NoCoin(5. + 6. - 3. - 2.),
//         );
//     }

//     #[test]
//     fn calculate_wallet_fees_rewards() {
//         let target = NodeId(0);
//         let mut blockchain = Blockchain(vec![
//             mock_block!(
//             2->0,f 1.,a 3.;
//             1->0,f 1.,a 0.;
//             3->4,f 1.,a 0.;
//             4->5,f 1.,a 0.| 0
//             ),
//             mock_block!(
//             2->0,f 2.,a 0.;
//             1->0,f 2.,a 0.;
//             5->1,f 2.,a 0.;
//             6->2,f 2.,a 0.| 1
//             ),
//         ]);
//         blockchain.0[0].transactions[0].as_mut().unwrap().from = None;

//         assert_eq_coin(
//             calculate_wallet(&target, &blockchain),
//             NoCoin(7.),
//         );
//     }

//     #[test]
//     fn calculate_all_wallets_properly() {
//         let mut blockchain = Blockchain(vec![
//             mock_block!(
//             0->1,f 1., a 5.;
//             2->1,f 2., a 3.| 2
//             ),
//             mock_block!(
//             2->0,f 1., a 2.;
//             2->1,f 2., a 3.| 1
//             ),
//             mock_block!(
//             1->2,f 1., a 5.;
//             2->0,f 2., a 3.| 0
//             )
//         ]);
//         blockchain.0[0].transactions[2] = Some(Transaction::new(None, NodeId(2), NoCoin(0.), NoCoin(10.)));
//         blockchain.0[1].transactions[2] = Some(Transaction::new(None, NodeId(1), NoCoin(0.), NoCoin(10.)));
//         blockchain.0[2].transactions[2] = Some(Transaction::new(None, NodeId(0), NoCoin(0.), NoCoin(10.)));

//         let wallets = calculate_all_wallets(&blockchain);

//         assert_eq_coin(NoCoin(12.), wallets[&NodeId(0)]);
//         assert_eq_coin(NoCoin(18.), wallets[&NodeId(1)]);
//         assert_eq_coin(NoCoin(0.), wallets[&NodeId(2)]);
//     }
// }
