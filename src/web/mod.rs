mod communication;
mod server;

pub use communication::{get_bitcoin_value_usd, get_chain, register_node};
pub use server::run;
