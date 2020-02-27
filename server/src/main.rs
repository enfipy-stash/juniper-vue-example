mod context;
mod database;
mod models;
mod schema;

use actix::{Actor, StreamHandler, ActorContext};
use actix_web::{
    get, http::{header, StatusCode}, post, web, App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_cors::Cors;
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

// #[juniper::graphql_subscription(Context = JuniperContext)]
// impl Subscription {
//     async fn subscribe_for_messages(ctx: &JuniperContext) -> Stream<Message> {
//         match ctx.ws_message {
//             Message::Connected(token) => {
//                 let user = parse_jwt_toke(token);
//                 chat.join_user(user);
//                 chat_usecase.subscribe_for_messages(user)
//             }
//             Message::Disconnected() => {
//                 chat.left_user(user);
//                 stream::empty::<Message>()
//             }
//         }
//     }
// }


struct JuniperWebSocketActor;

impl Actor for JuniperWebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("started");
        // let context = JuniperContext::init(database.get_ref().clone());
        // feature =
            // let stream = req.execute_subscribe(&graphql_root, &context, &ctx, token, [arguments])
            // while let Some(msg) = stream.next() {
                // ctx.text(msg)
                // }
        // self.spawn(feature)
    }
}

struct JuniperWebSocket; 
// {
//     user_id: Uuid,
//     request_ids_map: Map<i32, JuniperWebSocketActor>,
// }

impl Actor for JuniperWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("started");
        ctx.text(r#"{"lol":"O_O"}"#)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for JuniperWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        log::info!("msg: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                ctx.text(text)
                // let request_id = text.request_id;
                // create actor and add to map
                // with key: user_id, request_id
                // with value: JuniperWebSocketActor
                // start_abiter(start)
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(_)) => {
                // Iterate by self.request_ids_map
                // Remove all self.actors
                ctx.stop()
            },
            _ => (),
        }
    }

    // fn closed
        // Send message to every actor with message: Disconnected
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
    // let mut nstream = stream.into_inner();
    // let greq = web::Json::<GraphQLRequest>::from_request(&req, &mut nstream).await;
    log::info!("req: {:?}", req);
    ws::start_with_protocols(JuniperWebSocket, &["graphql-ws"], &req, stream)
    //log::info!("Sub: {:?}", res);
    //log::info!("greq: {:?}", greq);
    //log::info!("graphql_root: {:?}", graphql_root);
    //res
    //Ok(HttpResponse::build(StatusCode::OK))
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
    .bind("localhost:3000")?
    .run()
    .await
}
