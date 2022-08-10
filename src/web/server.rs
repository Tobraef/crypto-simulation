use std::{fmt::Display, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use actix_web::{
    middleware, route,
    web::{self, Data},
    HttpRequest, HttpResponse, Responder, get,
};

use anyhow::anyhow;

use crate::{
    domain::{
        acknowledge_node, try_add_block, try_add_transaction, try_create_node, Block,
        Network as DomainNetwork, Node, PubKey, Transaction,
    },
    web::communication::send_acknowledge_new_node,
};

#[derive(Debug)]
struct ErrResponse(anyhow::Error);

impl Display for ErrResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<anyhow::Error> for ErrResponse {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl actix_web::error::ResponseError for ErrResponse {}

type SNetwork = Data<Mutex<DomainNetwork>>;

pub struct Routes {
    pub new_block: &'static str,
    pub get_chain: &'static str,
    pub new_transaction: &'static str,
    pub acknowledge_new_node: &'static str,
    pub register: &'static str,
    pub get_pending_transactions: &'static str,
}

pub const ROUTES: Routes = Routes {
    new_block: "new_block",
    get_chain: "get_chain",
    new_transaction: "new_transaction",
    acknowledge_new_node: "acknowledge_new_node",
    register: "register",
    get_pending_transactions: "get_pending_transactions",
};

#[route("new_block", method = "POST")]
async fn new_block(
    network: SNetwork,
    block: web::Json<Block>,
) -> Result<impl Responder, ErrResponse> {
    let mut network = network.lock().await;
    try_add_block(&mut network, block.0)?;
    Ok(HttpResponse::Ok())
}

#[route("get_chain", method = "GET")]
async fn get_chain(network: SNetwork) -> impl Responder {
    let network = network.lock().await;
    serde_json::to_string(&network.blockchain)
}

#[get("get_pending_transactions")]
async fn get_pending_transactions(network: SNetwork) -> impl Responder {
    let network = network.lock().await;
    serde_json::to_string(&network.transactions_poll)
}

#[route("new_transaction", method = "POST")]
async fn new_transaction(
    transaction: web::Json<Transaction>,
    proof: web::Json<Vec<u8>>,
    network: SNetwork,
) -> Result<impl Responder, ErrResponse> {
    let mut network = network.lock().await;
    try_add_transaction(&mut network, transaction.0, proof.0)?;
    Ok(HttpResponse::Ok())
}

#[route("acknowledge_new_node", method = "POST")]
async fn acknowledge_new_node(
    node: web::Json<Node>,
    network: SNetwork,
) -> Result<impl Responder, ErrResponse> {
    let mut network = network.lock().await;
    acknowledge_node(&mut network, node.0)?;
    Ok(HttpResponse::Ok())
}

#[route("register", method = "GET")]
async fn register(
    req: HttpRequest,
    key: web::Json<PubKey>,
    network: SNetwork,
    client: Data<reqwest::Client>,
) -> Result<impl Responder, ErrResponse> {
    let mut network = network.lock().await;
    let address = req
        .peer_addr()
        .ok_or(anyhow!("Couldn't get address from the request. Rejected."))?;
    let node = try_create_node(&mut network, address, key.0)?;
    send_acknowledge_new_node(client.as_ref(), &node.clone(), &network.nodes).await?;
    Ok(web::Json(network.nodes.clone()))
}

pub async fn run(addr: SocketAddr, network: Arc<Mutex<DomainNetwork>>) -> anyhow::Result<()> {
    log::info!("Starting server on {:?}", addr);
    let network = Data::from(network);
    let client = Data::new(reqwest::Client::new());
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(network.clone())
            .app_data(client.clone())
            .service(new_transaction)
            .service(register)
            .service(acknowledge_new_node)
            .service(self::new_block)
            .service(self::get_chain)
            .service(self::get_pending_transactions)
            .wrap(middleware::Logger::default())
    })
    .bind(addr)?
    .run()
    .await?;
    Ok(())
}
