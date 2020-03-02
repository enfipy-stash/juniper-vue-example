use crate::websocket::WebSocketSession;
use actix::{prelude::*, Actor};
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::HashMap;

pub struct WebSocketServer {
    sessions: HashMap<usize, Addr<WebSocketSession>>,
    rng: ThreadRng,
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;
}

/// New ws session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Addr<WebSocketSession>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

impl Handler<Connect> for WebSocketServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}
