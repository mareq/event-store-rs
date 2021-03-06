//! Postgres-backed cache and store adapters

mod cache;
mod store;

pub use self::cache::PgCacheAdapter;
pub use self::store::PgStoreAdapter;
use r2d2::Pool;
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::PostgresConnectionManager;
use sha2::{Digest, Sha256};

use StoreQuery;

type Connection = Pool<PostgresConnectionManager>;

/// Representation of a Postgres query and args
#[derive(Debug)]
pub struct PgQuery {
    /// Query string with placeholders
    pub query: String,

    /// Arguments to use for the query
    pub args: Vec<Box<ToSql>>,
}

impl StoreQuery for PgQuery {
    fn unique_id(&self) -> String {
        let hash = Sha256::digest(format!("{:?}:[{}]", self.args, self.query).as_bytes());
        hash.iter().fold(String::new(), |mut acc, hex| {
            acc.push_str(&format!("{:X}", hex));
            acc
        })
    }
}

impl PgQuery {
    /// Create a new query from a query string and arguments
    pub fn new(query: &str, args: Vec<Box<ToSql>>) -> Self {
        Self {
            query: query.into(),
            args,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn gets_unique_id() {
        let pg = PgQuery::new("whatever", vec![]);
        assert_eq!(
            pg.unique_id(),
            "5C91F3755337FAF226A3D3BB2C3B0EF1D1699C3B5CA6272D0858C78FFB244FB"
        );
    }
}
