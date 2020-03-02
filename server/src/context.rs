use crate::database::Database;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub struct GraphqlContext {
    database: Arc<Database>,
    received_ws_close_signal: Arc<AtomicBool>,
}

impl GraphqlContext {
    pub fn init(database: Arc<Database>) -> Self {
        Self {
            database,
            received_ws_close_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn database(&self) -> Arc<Database> {
        self.database.clone()
    }

    pub fn received_ws_close_signal(&self) -> Arc<AtomicBool> {
        self.received_ws_close_signal.clone()
    }

    pub fn send_ws_close_signal(&self) {
        // Todo: read more about ordering
        self.received_ws_close_signal.store(true, Ordering::Relaxed);
    }
}

impl juniper::Context for GraphqlContext {}
