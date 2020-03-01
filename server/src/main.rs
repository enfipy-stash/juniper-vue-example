mod context;
mod database;
mod models;
mod schema;

use actix::prelude::*;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_cors::Cors;
use actix_web::{
    get,
    http::{header, StatusCode},
    post, web, App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use context::JuniperContext;
use database::Database;
use juniper::{
    http::{playground::playground_source, GraphQLRequest, GraphQLResponse, StreamError, StreamGraphQLResponse},
    DefaultScalarValue, InputValue,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::stream::StreamExt;

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
    let context = JuniperContext::init(database.get_ref().clone());
    let res = req.execute_async(&graphql_root, &context).await;
    let json_res = serde_json::to_string(&res)?;
    Ok(HttpResponse::Ok().content_type("application/json").body(json_res))
}

#[derive(Debug, Deserialize)]
#[serde(bound = "GraphQLPayload<S>: Deserialize<'de>")]
struct WsPayload<S = DefaultScalarValue> {
    id: Option<String>,
    #[serde(rename(deserialize = "type"))]
    type_name: String,
    payload: Option<GraphQLPayload<S>>,
}

#[derive(Debug, Deserialize)]
#[serde(bound = "InputValue<S>: Deserialize<'de>")]
struct GraphQLPayload<S = DefaultScalarValue> {
    variables: Option<InputValue<S>>,
    extensions: Option<HashMap<String, String>>,
    #[serde(rename(deserialize = "operationName"))]
    operaton_name: Option<String>,
    query: Option<String>,
}

pub struct WebSocket {
    graphql_root: Arc<schema::Schema>,
    database: Arc<Database>,
}

impl WebSocket {
    pub fn new(graphql_root: Arc<schema::Schema>, database: Arc<Database>) -> Self {
        Self { graphql_root, database }
    }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;
}

pub struct JuniperResponce {
    json: String,
}

impl Message for JuniperResponce {
    type Result = ();
}

impl Handler<JuniperResponce> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: JuniperResponce, ctx: &mut Self::Context) {
        log::info!("{}", msg.json);
        ctx.text(msg.json);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let request = serde_json::from_str::<WsPayload>(&text).unwrap();
                match request.type_name.as_str() {
                    "connection_init" => {}
                    "start" => {
                        let payload = request.payload.expect("could not deserialize payload");
                        // let request_id = request.id.unwrap_or("1".to_owned());
                        let database = self.database.clone();
                        let graphql_root = self.graphql_root.clone();
                        let context = JuniperContext::init(database);
                        let req = GraphQLRequest::new(payload.query.unwrap(), payload.operaton_name, payload.variables);

                        let addr = ctx.address();
                        ctx.spawn(
                            async move {
                                let res = req.subscribe(&graphql_root, &context).await;
                                let mut stream = res.into_stream().unwrap();
                                while let Some(response) = stream.next().await {
                                    let json = serde_json::to_string(&response).unwrap();
                                    addr.do_send(JuniperResponce { json });
                                }
                            }
                            .into_actor(self),
                        );
                    }
                    "stop" => {}
                    _ => {}
                }
            }
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => (),
        }
    }
}

#[get("/subscriptions")]
async fn subscriptions_handler(
    req: HttpRequest,
    stream: web::Payload,
    graphql_root: web::Data<Arc<schema::Schema>>,
    database: web::Data<Arc<Database>>,
) -> Result<impl Responder, Error> {
    ws::start_with_protocols(
        WebSocket::new(graphql_root.get_ref().clone(), database.get_ref().clone()),
        &["graphql-ws"],
        &req,
        stream,
    )
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug,actix_server=info,actix_web=trace");
    env_logger::init();

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
