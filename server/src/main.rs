mod context;
mod database;
mod models;
mod schema;

use actix::{Actor, StreamHandler};
use actix_web::{
    get, http::StatusCode, post, web, App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use context::JuniperContext;
use database::Database;
use juniper::http::{playground::playground_source, GraphQLRequest};
use std::sync::Arc;
use tokio::stream::{Stream, StreamExt};

#[get("/playground")]
pub async fn playground_handler() -> impl Responder {
    let html = playground_source("/graphql", Some("/subscriptions"));
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

#[post("/graphql")]
async fn graphql_handler(
    graphql_root: web::Data<Arc<schema::Schema>>,
    req: web::Json<GraphQLRequest>,
    database: web::Data<Arc<Database>>,
) -> Result<impl Responder, Error> {
    let res = web::block(move || {
        let context = JuniperContext::init(database.get_ref().clone());
        let res = req.execute(&graphql_root, &context);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok().content_type("application/json").body(res))
}

struct JuniperWebSocket;

impl Actor for JuniperWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("started");
        ctx.text(r#"{{"lol":"O_O"}}"#)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for JuniperWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        log::info!("msg: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/subscriptions")]
async fn subscriptions_handler(
    req: HttpRequest,
    stream: web::Payload,
    //graphql_root: web::Data<Arc<schema::Schema>>,
) -> Result<impl Responder, Error> {
    //let payload = stream.into_inner();
    // let mut bytes = web::BytesMut::new();
    // log::info!("lol");
    // while let Some(item) = payload.next().await {
    //     bytes.extend_from_slice(&item?);
    // }
    // log::info!("Body {:?}!", bytes);
    //log::info!("req: {:?}", req);
    ws::start(JuniperWebSocket, &req, stream)
    //log::info!("Sub: {:?}", res);
    // let mut stream = stream.into_inner();
    // let greq = web::Json::<GraphQLRequest>::from_request(&req, &mut stream).await;
    //log::info!("greq: {:?}", greq);
    //log::info!("graphql_root: {:?}", graphql_root);
    //res
    //Ok(HttpResponse::build(StatusCode::OK))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug,actix_server=info,actix_web=debug");
    env_logger::init();

    let graphql_root = Arc::new(schema::init());
    let database = Arc::new(Database::new());

    HttpServer::new(move || {
        App::new()
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
