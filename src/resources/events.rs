// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for Talos event streaming.
//!
//! This module provides types for subscribing to Talos cluster events
//! via the Events API, which streams machine state changes, service
//! events, and other cluster activity in real-time.
//!
//! # Example
//!
//! ```rust,no_run
//! use talos_api_rs::{TalosClient, TalosClientConfig};
//! use talos_api_rs::resources::{EventsRequest, Event};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = TalosClientConfig::builder("https://10.0.0.1:50000")
//!     .insecure()
//!     .build();
//! let client = TalosClient::new(config).await?;
//!
//! // Get the last 10 events
//! let events = client.events(EventsRequest::tail(10)).await?;
//! for event in events {
//!     println!("Event {}: actor={}", event.id, event.actor_id);
//! }
//! # Ok(())
//! # }
//! ```

use crate::api::generated::machine::{Event as ProtoEvent, EventsRequest as ProtoEventsRequest};

// =============================================================================
// EventsRequest
// =============================================================================

/// Request to subscribe to Talos events.
///
/// Events provide real-time visibility into cluster state changes,
/// service lifecycle, and machine activity.
#[derive(Debug, Clone, Default)]
pub struct EventsRequest {
    /// Number of past events to return before streaming new ones.
    pub tail_events: i32,
    /// Start streaming from a specific event ID.
    pub tail_id: String,
    /// Return events from the last N seconds.
    pub tail_seconds: i32,
    /// Filter events by actor ID.
    pub with_actor_id: String,
}

impl EventsRequest {
    /// Create a new events request with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a request to get the last N events.
    #[must_use]
    pub fn tail(count: i32) -> Self {
        Self {
            tail_events: count,
            ..Default::default()
        }
    }

    /// Create a request to get events from the last N seconds.
    #[must_use]
    pub fn since_seconds(seconds: i32) -> Self {
        Self {
            tail_seconds: seconds,
            ..Default::default()
        }
    }

    /// Create a request starting from a specific event ID.
    #[must_use]
    pub fn since_id(id: impl Into<String>) -> Self {
        Self {
            tail_id: id.into(),
            ..Default::default()
        }
    }

    /// Set the number of past events to return.
    #[must_use]
    pub fn with_tail_events(mut self, count: i32) -> Self {
        self.tail_events = count;
        self
    }

    /// Start streaming from a specific event ID.
    #[must_use]
    pub fn with_tail_id(mut self, id: impl Into<String>) -> Self {
        self.tail_id = id.into();
        self
    }

    /// Return events from the last N seconds.
    #[must_use]
    pub fn with_tail_seconds(mut self, seconds: i32) -> Self {
        self.tail_seconds = seconds;
        self
    }

    /// Filter events by actor ID.
    #[must_use]
    pub fn with_actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.with_actor_id = actor_id.into();
        self
    }
}

impl From<EventsRequest> for ProtoEventsRequest {
    fn from(req: EventsRequest) -> Self {
        Self {
            tail_events: req.tail_events,
            tail_id: req.tail_id,
            tail_seconds: req.tail_seconds,
            with_actor_id: req.with_actor_id,
        }
    }
}

// =============================================================================
// Event
// =============================================================================

/// A Talos cluster event.
///
/// Events represent state changes and activities in the Talos cluster,
/// such as service starts/stops, configuration changes, and machine
/// state transitions.
#[derive(Debug, Clone)]
pub struct Event {
    /// Node that generated the event.
    pub node: Option<String>,
    /// Unique event identifier.
    pub id: String,
    /// Actor that triggered the event.
    pub actor_id: String,
    /// Event payload as protobuf Any type.
    ///
    /// The type URL indicates the specific event type, and the value
    /// contains the serialized event data.
    pub data: Option<EventData>,
}

/// Event payload data.
#[derive(Debug, Clone)]
pub struct EventData {
    /// Type URL identifying the event type (e.g., "talos/runtime/MachineStatusEvent").
    pub type_url: String,
    /// Serialized event payload.
    pub value: Vec<u8>,
}

impl Event {
    /// Check if this event has data.
    #[must_use]
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    /// Get the event type from the data type URL.
    ///
    /// Returns the short type name (e.g., "MachineStatusEvent") or None
    /// if no data is present.
    #[must_use]
    pub fn event_type(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|d| d.type_url.rsplit('/').next())
    }
}

impl From<ProtoEvent> for Event {
    fn from(proto: ProtoEvent) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            id: proto.id,
            actor_id: proto.actor_id,
            data: proto.data.map(|any| EventData {
                type_url: any.type_url,
                value: any.value,
            }),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_request_new() {
        let req = EventsRequest::new();
        assert_eq!(req.tail_events, 0);
        assert!(req.tail_id.is_empty());
        assert_eq!(req.tail_seconds, 0);
        assert!(req.with_actor_id.is_empty());
    }

    #[test]
    fn test_events_request_tail() {
        let req = EventsRequest::tail(100);
        assert_eq!(req.tail_events, 100);
    }

    #[test]
    fn test_events_request_since_seconds() {
        let req = EventsRequest::since_seconds(300);
        assert_eq!(req.tail_seconds, 300);
    }

    #[test]
    fn test_events_request_since_id() {
        let req = EventsRequest::since_id("event-123");
        assert_eq!(req.tail_id, "event-123");
    }

    #[test]
    fn test_events_request_builder() {
        let req = EventsRequest::new()
            .with_tail_events(50)
            .with_tail_seconds(60)
            .with_actor_id("my-actor");

        assert_eq!(req.tail_events, 50);
        assert_eq!(req.tail_seconds, 60);
        assert_eq!(req.with_actor_id, "my-actor");
    }

    #[test]
    fn test_events_request_to_proto() {
        let req = EventsRequest::tail(25)
            .with_tail_id("id-456")
            .with_actor_id("actor-1");

        let proto: ProtoEventsRequest = req.into();
        assert_eq!(proto.tail_events, 25);
        assert_eq!(proto.tail_id, "id-456");
        assert_eq!(proto.with_actor_id, "actor-1");
    }

    #[test]
    fn test_event_from_proto() {
        use crate::api::generated::common::Metadata;

        let proto = ProtoEvent {
            metadata: Some(Metadata {
                hostname: "node-1".to_string(),
                error: String::new(),
                status: None,
            }),
            id: "event-001".to_string(),
            actor_id: "kubelet".to_string(),
            data: Some(prost_types::Any {
                type_url: "talos/runtime/MachineStatusEvent".to_string(),
                value: vec![1, 2, 3],
            }),
        };

        let event = Event::from(proto);
        assert_eq!(event.node, Some("node-1".to_string()));
        assert_eq!(event.id, "event-001");
        assert_eq!(event.actor_id, "kubelet");
        assert!(event.has_data());
        assert_eq!(event.event_type(), Some("MachineStatusEvent"));
    }

    #[test]
    fn test_event_without_data() {
        let proto = ProtoEvent {
            metadata: None,
            id: "event-002".to_string(),
            actor_id: "".to_string(),
            data: None,
        };

        let event = Event::from(proto);
        assert!(event.node.is_none());
        assert!(!event.has_data());
        assert!(event.event_type().is_none());
    }

    #[test]
    fn test_event_type_extraction() {
        let data = EventData {
            type_url: "type.googleapis.com/talos.runtime.MachineStatusEvent".to_string(),
            value: vec![],
        };

        let event = Event {
            node: None,
            id: "test".to_string(),
            actor_id: "".to_string(),
            data: Some(data),
        };

        // Returns the full type name after the last '/'
        assert_eq!(event.event_type(), Some("talos.runtime.MachineStatusEvent"));
    }
}
