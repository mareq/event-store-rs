//! Stub emitter implementation

use super::EmitterAdapter;
use event::Event;
use event_store_derive_internals::EventData;
use serde_json::Value as JsonValue;
use std::io::Error;
use std::thread::{self, JoinHandle};

/// Stub event emitter
#[derive(Clone)]
pub struct StubEmitterAdapter {}

impl StubEmitterAdapter {
    /// Create a new emitter stub
    pub fn new() -> Self {
        Self {}
    }
}

impl EmitterAdapter for StubEmitterAdapter {
    fn emit<E: EventData>(&self, _event: &Event<E>) -> Result<(), Error> {
        Ok(())
    }

    fn emit_with_string_ident(
        &self,
        _event_namespace: &str,
        _event_type: &str,
        _event: &JsonValue,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn subscribe<ED, H>(&self, _handler: H) -> JoinHandle<()>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> (),
    {
        thread::spawn(move || {
            println!("Stub subscribe");
        })
    }
}
