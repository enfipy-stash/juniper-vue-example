use crate::database::Database;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct JuniperContext {
    database: Arc<Database>,
}

impl JuniperContext {
    pub fn init(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn database(&self) -> Arc<Database> {
        self.database.clone()
    }
}

impl juniper::Context for JuniperContext {}
