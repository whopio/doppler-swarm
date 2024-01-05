use bytes::Bytes;
use serde::Deserialize;

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

pub fn parse_watch_event(raw_payload: &Bytes) -> crate::result::Result<WatchEvent> {
    if !raw_payload.starts_with(b"event: message\ndata:") {
        return Err("invalid event".into());
    }

    let event: EventPayload = serde_json::from_slice(&raw_payload[20..]).map_err(|e| {
        format!(
            "Failed to parse event payload: {}. Payload: {:?}",
            e, raw_payload
        )
    })?;

    Ok(event.event_type)
}
