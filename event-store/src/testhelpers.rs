//! Test helpers. Do not use in application code.

use adapters::PgQuery;
use prelude::*;
use r2d2::{self, Pool};
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use redis::Client as RedisClient;
use std::time::{SystemTime, UNIX_EPOCH};
use Event;

/// Test event
#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestIncrementEvent {
    /// Increment by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

/// Test event
#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestDecrementEvent {
    /// Decrement by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

#[derive(Events, Debug)]
/// Set of all events in the domain
pub enum TestEvents {
    /// Increment
    Inc(Event<TestIncrementEvent>),
    /// Decrement
    Dec(Event<TestDecrementEvent>),
}

// impl EventData for TestEvents {}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
/// Testing entity for a pretend domain
pub struct TestCounterEntity {
    /// Current counter value
    pub counter: i32,
}

impl Default for TestCounterEntity {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl Aggregator<TestEvents, String, PgQuery> for TestCounterEntity {
    fn apply_event(acc: Self, event: &TestEvents) -> Self {
        let counter = match event {
            TestEvents::Inc(ref inc) => acc.counter + inc.data.by,
            TestEvents::Dec(ref dec) => acc.counter - dec.data.by,
        };

        Self { counter, ..acc }
    }

    fn query(field: String) -> PgQuery {
        let mut params: Vec<Box<ToSql>> = Vec::new();

        params.push(Box::new(field));

        PgQuery::new("select * from events where data->>'ident' = $1", params)
    }
}

/// Connect to a local Postgres database on port 5430
pub fn pg_connect() -> Pool<PostgresConnectionManager> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    pool
}

/// Connect to a Redis server on port 6378
pub fn redis_connect() -> RedisClient {
    redis::Client::open("redis://127.0.0.1:6378").expect("Could not connect to Redis server")
}

/// Create an event store from a Postgres connection pool
#[macro_export]
macro_rules! pg_store {
    ($conn:ident) => {{
        let store_adapter = PgStoreAdapter::new($conn.clone());
        let cache_adapter = PgCacheAdapter::new($conn.clone());
        let emitter_adapter = StubEmitterAdapter::new();

        EventStore::new(store_adapter, cache_adapter, emitter_adapter)
    }};
}

/// Create an event store from a Postgres connection pool with a Redis cache
#[macro_export]
macro_rules! pg_store_with_redis_cache {
    ($conn:ident, $redis_conn:ident) => {{
        let store_adapter = PgStoreAdapter::new($conn.clone());
        let cache_adapter = RedisCacheAdapter::new($redis_conn.clone());
        let emitter_adapter = StubEmitterAdapter::new();

        EventStore::new(store_adapter, cache_adapter, emitter_adapter)
    }};
}

/// Delete all `TestEvent` events matching an identifier from the `events` table
#[macro_export]
macro_rules! pg_delete_events {
    ($conn:ident, $ident:expr) => {{
        $conn
            .get()
            .unwrap()
            .execute("DELETE FROM events WHERE data->>'ident' = $1", &[&$ident])
            .expect("Failed to delete events");
    }};
}

/// Remove all events from the database
#[macro_export]
macro_rules! pg_empty_events {
    ($conn:ident) => {{
        $conn
            .get()
            .unwrap()
            .execute("TRUNCATE events", &[])
            .expect("Failed to delete events");
    }};
}

/// Remove EVERY entry from the `aggregate_cache` table
#[macro_export]
macro_rules! pg_empty_cache {
    ($conn:ident) => {{
        $conn
            .get()
            .unwrap()
            .execute("TRUNCATE aggregate_cache", &[])
            .expect("Failed to trunacte cache table");
    }};
}

/// Remove every entry from the Redis cache
#[macro_export]
macro_rules! redis_empty_cache {
    ($conn:ident) => {{
        redis::cmd("FLUSHDB").execute(&$conn);
    }};
}

fn current_time_ms() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let in_ms =
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    in_ms
}

/// Create a new database with a random name, returning the connection
pub fn pg_create_random_db() -> Pool<PostgresConnectionManager> {
    let db_id = format!("eventstorerust-test-{}", current_time_ms());

    println!("Create test DB {}", db_id);

    let manager =
        PostgresConnectionManager::new("postgres://postgres@localhost:5430", TlsMode::None)
            .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    let conn = pool.get().unwrap();

    conn.batch_execute(&format!("CREATE DATABASE \"{}\"", db_id))
        .unwrap();

    let manager = PostgresConnectionManager::new(
        format!("postgres://postgres@localhost:5430/{}", db_id),
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    let conn = pool.get().unwrap();

    conn.batch_execute(
        r#"
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

        CREATE TABLE events (
            id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
            data jsonb NOT NULL,
            context jsonb DEFAULT '{}'::jsonb
        );

        CREATE TABLE aggregate_cache (
            id VARCHAR(64) PRIMARY KEY,
            data jsonb NOT NULL,
            time timestamp without time zone DEFAULT now()
        );
    "#,
    )
    .unwrap();

    pool
}
