// use std::net::{Ipv4Addr, SocketAddrV4};

// use super::network::{Node, NodeId};

// impl Default for Node {
//     fn default() -> Self {
//         let mut rng = rand::thread_rng();
//         Self {
//             id: NodeId(0),
//             addr: SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 0),
//             pub_key: super::PubKey(
//                 rsa::RsaPrivateKey::new(&mut rng, PRIVATE_KEY_LEN)
//                     .unwrap()
//                     .to_public_key(),
//             ),
//         }
//     }
// }

// impl Default for User {
//     fn default() -> Self {
//         let mut rng = rand::thread_rng();
//         Self {
//             node: Default::default(),
//             priv_key: super::PrivKey(rsa::RsaPrivateKey::new(&mut rng, PRIVATE_KEY_LEN).unwrap()),
//         }
//     }
// }

// impl Default for BlockHeader {
//     fn default() -> Self {
//         Self {
//             hash: Default::default(),
//             prev_hash: Default::default(),
//             index: BlockIndex(0),
//             timestamp: 0,
//         }
//     }
// }

// impl Default for BlockHash {
//     fn default() -> Self {
//         Self([0; 64])
//     }
// }

// impl Default for Block {
//     fn default() -> Self {
//         Self {
//             header: BlockHeader {
//                 ..Default::default()
//             },
//             mined_by: NodeId(usize::MAX),
//             transactions: Default::default(),
//         }
//     }
// }

// impl Clone for Transaction {
//     fn clone(&self) -> Self {
//         Self {
//             from: self.from.clone(),
//             to: self.to.clone(),
//             fee: self.fee.clone(),
//             ammount: self.ammount.clone(),
//         }
//     }
// }

// pub fn assert_eq_coin(a: NoCoin, b: NoCoin) {
//     assert!(
//         (a - b).0.abs() < 0.0001,
//         "Dif between {:?} and {:?} too much",
//         a,
//         b
//     )
// }

// #[macro_export]
// macro_rules! mock_block {
//     ($($e1:literal->$e2:literal,f $e3:literal,a $e4:literal);*| $miner:literal) => {{
//         let mut block = Block::default();
//         block.mined_by = NodeId($miner);
//         let mut i = 0;
//         $(
//             block.transactions[i] = Some(Transaction::new(
//                 Some(NodeId($e1)),
//                 NodeId($e2),
//                 NoCoin($e3),
//                 NoCoin($e4),
//             ));

//             i += 1;
//         )*
//         let _ = i;
//         block
//     }};
// }
