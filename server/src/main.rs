mod context;
mod database;
mod models;
mod schema;
mod websocket;

use actix::prelude::*;
use actix::Actor;
use actix_cors::Cors;
use actix_web::{get, http::header, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use context::GraphqlContext;
use database::Database;
use juniper::http::{playground::playground_source, GraphQLRequest};
use schema::GraphqlRoot;
use std::sync::Arc;
use websocket::{WebSocketServer, WebSocketSession};

#[get("/playground")]
pub async fn playground_handler() -> impl Responder {
    let html = playground_source("/graphql", Some("/subscriptions"));
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

#[post("/graphql")]
async fn graphql_handler(
    graphql_root: web::Data<Arc<GraphqlRoot>>,
    req: web::Json<GraphQLRequest>,
    database: web::Data<Arc<Database>>,
) -> Result<impl Responder, Error> {
    let context = GraphqlContext::init(database.get_ref().clone());
    let res = req.execute_async(&graphql_root, &context).await;
    let json_res = serde_json::to_string(&res)?;
    Ok(HttpResponse::Ok().content_type("application/json").body(json_res))
}

#[get("/subscriptions")]
async fn subscriptions_handler(
    req: HttpRequest,
    stream: web::Payload,
    ws_server: web::Data<Addr<WebSocketServer>>,
    graphql_root: web::Data<Arc<GraphqlRoot>>,
    database: web::Data<Arc<Database>>,
) -> Result<impl Responder, Error> {
    let context = GraphqlContext::init(database.get_ref().clone());
    ws::start_with_protocols(
        WebSocketSession::new(graphql_root.get_ref().clone(), context, ws_server.get_ref().clone()),
        &["graphql-ws"],
        &req,
        stream,
    )
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug,actix_server=info,actix_web=trace");
    env_logger::init();

    let ws_server = WebSocketServer::default().start();
    let graphql_root = Arc::new(schema::init());
    let database = Arc::new(Database::new());

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600)
                    .finish(),
            )
            .data(ws_server.clone())
            .data(graphql_root.clone())
            .data(database.clone())
            .service(graphql_handler)
            .service(playground_handler)
            .service(subscriptions_handler)
    })
    .bind("localhost:8080")?
    .run()
    .await
}
