use super::server::*;
use crate::{context::GraphqlContext, schema::GraphqlRoot};
use actix::{prelude::*, Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use juniper::http::GraphQLRequest;
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::stream::StreamExt;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Deserialize, Serialize)]
struct WsPayload {
    #[serde(rename = "type")]
    type_name: String,
    id: Option<String>,
    payload: serde_json::Value,
}

#[derive(Message)]
#[rtype(result = "()")]
struct WsGraphql(WsPayload);

#[derive(Message)]
#[rtype(result = "()")]
struct WsError(WsPayload);

impl WsPayload {
    pub fn to_graphql_request(self) -> Result<GraphQLRequest, WsError> {
        let id = self.id.clone();
        serde_json::from_value(self.payload).map_err(|err| WsError::from_str(id, &err.to_string()))
    }
}

impl WsGraphql {
    pub fn from_value(request_id: Option<String>, value: serde_json::Value) -> Self {
        WsGraphql(WsPayload {
            type_name: "data".to_owned(),
            id: request_id,
            payload: value,
        })
    }
}

impl WsError {
    pub fn from_value(request_id: Option<String>, value: serde_json::Value) -> Self {
        WsError(WsPayload {
            type_name: "error".to_owned(),
            id: request_id,
            payload: value,
        })
    }

    pub fn from_str(request_id: Option<String>, error: &str) -> Self {
        WsError(WsPayload {
            type_name: "error".to_owned(),
            id: request_id,
            payload: serde_json::Value::String(error.to_owned()),
        })
    }
}

impl From<WsGraphql> for WsPayload {
    fn from(ws_graphql: WsGraphql) -> WsPayload {
        ws_graphql.0
    }
}

impl From<WsError> for WsPayload {
    fn from(ws_error: WsError) -> Self {
        ws_error.0
    }
}

pub struct WebSocketSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    graphql_root: Arc<GraphqlRoot>,
    graphql_context: GraphqlContext,
    server_addr: Addr<WebSocketServer>,
}

impl WebSocketSession {
    pub fn new(
        graphql_root: Arc<GraphqlRoot>,
        graphql_context: GraphqlContext,
        server_addr: Addr<WebSocketServer>,
    ) -> Self {
        Self {
            id: 0,
            hb: Instant::now(),
            graphql_root,
            graphql_context,
            server_addr,
        }
    }

    /// helper method that sends ping to client every second.
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // notify ws server
                act.server_addr.do_send(Disconnect { id: act.id });
                log::info!("hb close");
                // notify graphql_context. Todo: check if its works
                act.graphql_context.send_ws_close_signal();
                // stop actor
                ctx.stop();
                // don't try to send a ping
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        let addr = ctx.address();
        self.server_addr
            .send(Connect { addr })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with ws server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify ws server
        self.server_addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<WsGraphql> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: WsGraphql, ctx: &mut Self::Context) {
        let json = serde_json::to_string(&msg.0).unwrap();
        log::info!("WsGraphql: {}", json);
        ctx.text(json);
    }
}

impl Handler<WsError> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: WsError, ctx: &mut Self::Context) {
        let json = serde_json::to_string(&msg.0).unwrap();
        //log::info!("WsError: {}", json);
        ctx.text(json);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
        let addr = ctx.address();
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let ws_payload = match serde_json::from_str::<WsPayload>(&text) {
                    Ok(r) => r,
                    Err(error) => {
                        addr.do_send(WsError::from_str(None, &error.to_string()));
                        return;
                    }
                };
                let request_id = ws_payload.id.clone();

                match ws_payload.type_name.as_str() {
                    "connection_init" => {
                        log::info!("connection_init: {}", text);
                    }
                    "start" => {
                        log::info!("start: {}", text);

                        let graphql_root = self.graphql_root.clone();
                        let graphql_context = self.graphql_context.clone();
                        let req = match ws_payload.to_graphql_request() {
                            Ok(r) => r,
                            Err(error) => {
                                addr.do_send(error);
                                return;
                            }
                        };

                        ctx.spawn(
                            async move {
                                let res = req.subscribe(&graphql_root, &graphql_context).await;
                                let mut stream = match res.into_stream() {
                                    Ok(s) => s,
                                    Err(error) => {
                                        addr.do_send(WsError::from_value(
                                            request_id.clone(),
                                            serde_json::to_value(&error).unwrap(),
                                        ));
                                        // Todo: send message that we are closing channel (ws_complete)
                                        // Todo: close channel
                                        return;
                                    }
                                };
                                while let Some(response) = stream.next().await {
                                    addr.do_send(WsGraphql::from_value(
                                        request_id.clone(),
                                        serde_json::to_value(&response).unwrap(),
                                    ));
                                }
                            }
                            .into_actor(self),
                        );
                    }
                    "stop" => {
                        log::info!("stop: {}", text);
                        self.graphql_context.send_ws_close_signal();
                        ctx.stop();
                    }
                    _ => {
                        log::info!("unknown: {}", text);
                    }
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                log::info!("close");
                // notify graphql_context. Todo: check if its works
                self.graphql_context.send_ws_close_signal();
                ctx.stop();
            }
            _ => (),
        }
    }
}
