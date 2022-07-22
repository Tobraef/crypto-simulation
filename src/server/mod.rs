use std::net::SocketAddr;

use actix_web::{route, HttpRequest, Responder};

#[route("register", method = "GET")]
async fn register(req: HttpRequest) -> impl Responder {
    actix_web::HttpResponse::Ok()
}

pub async fn run(addr: SocketAddr) -> anyhow::Result<()> {
    actix_web::HttpServer::new(|| actix_web::App::new().service(register))
        .bind(addr)?
        .run()
        .await
        .map_err(|e| e.into())
}
