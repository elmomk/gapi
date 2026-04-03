mod users;
mod data;
mod webhooks;
mod intraday;
mod extended;

use crate::db::DbPool;

pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}
