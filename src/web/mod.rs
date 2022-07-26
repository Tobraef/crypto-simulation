mod communication;
mod server;

pub use server::run;
pub use communication::{register_node, get_bitcoin_value_usd, get_chain};
