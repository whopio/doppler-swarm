use bytes::Bytes;
use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct EventPayload {
    #[serde(rename = "type")]
    pub event_type: WatchEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum WatchEvent {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "secrets.update")]
    SecretsUpdate,
}

pub fn parse_watch_event(raw_payload: &Bytes) -> Result<WatchEvent, Error> {
    if !raw_payload.starts_with(b"event: message\ndata:") {
        return Err("invalid event".into());
    }

    let event: EventPayload = serde_json::from_slice(&raw_payload[20..])
        .map_err(|e| format!("failed to parse event payload: {e}"))?;

    Ok(event.event_type)
}
