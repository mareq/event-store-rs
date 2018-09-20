extern crate event_store;
extern crate tokio;

use event_store::adapters::{StubCacheAdapter, StubEmitterAdapter, StubStoreAdapter};
use event_store::prelude::*;
use event_store::testhelpers::*;
use event_store::{Event, EventStore};
use tokio::executor::current_thread::block_on_all;

#[test]
fn create_store() {
    let store_adapter = StubStoreAdapter::new();
    let cache_adapter = StubCacheAdapter::new();
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    let sub = store.subscribe(|_e: &Event<TestIncrementEvent>, st| {
        let some_aggregate: TestCounterEntity =
            block_on_all(st.aggregate(String::from("some_query_param")))
                .expect("Could not aggregate example event");

        println!("SOME AGG {:?}", some_aggregate);
    });

    block_on_all(sub).expect("Sth");
}
