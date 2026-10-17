use actix_web::{HttpRequest, HttpResponse, web};
use compact_str::CompactString;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Realtime message payloads matching upstream `frappe.realtime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "event", content = "data")]
pub enum RealtimeEvent {
    #[serde(rename = "doc_update")]
    DocUpdate {
        doctype: CompactString,
        name: CompactString,
        modified: String,
        user: CompactString,
    },
    #[serde(rename = "task_progress")]
    TaskProgress {
        task_id: CompactString,
        progress: u8,
        description: CompactString,
    },
    #[serde(rename = "user_presence")]
    UserPresence {
        user: CompactString,
        doctype: CompactString,
        name: CompactString,
        status: CompactString,
    },
    #[serde(rename = "notification")]
    Notification {
        user: CompactString,
        title: CompactString,
        message: CompactString,
    },
}

/// Realtime broadcast frame containing target room and event payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealtimeBroadcast {
    pub room: CompactString,
    pub event: RealtimeEvent,
}

/// Error occurred during lock-free broadcast dispatch.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Broadcast channel closed or full: {0}")]
pub struct BroadcastError(pub CompactString);

/// In-memory Realtime Hub managing room subscriptions (`doc:<doctype>:<name>`, `user:<user>`).
/// Uses a lock-free Tokio broadcast channel for zero-contention, low-latency event distribution.
#[derive(Debug, Clone)]
pub struct RealtimeHub {
    // Room -> set of session IDs
    rooms: Arc<RwLock<HashMap<CompactString, HashSet<CompactString>>>>,
    // Session ID -> set of rooms subscribed
    subscriptions: Arc<RwLock<HashMap<CompactString, HashSet<CompactString>>>>,
    // Lock-free broadcast channel for zero-contention distribution
    broadcast_tx: tokio::sync::broadcast::Sender<RealtimeBroadcast>,
}

impl Default for RealtimeHub {
    fn default() -> Self {
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(4096);
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }
}

impl RealtimeHub {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Dispatches an event to all subscribers of a room via the lock-free broadcast channel.
    pub fn dispatch(&self, room: &str, event: RealtimeEvent) -> Result<usize, BroadcastError> {
        self.broadcast_tx
            .send(RealtimeBroadcast {
                room: CompactString::from(room),
                event,
            })
            .map_err(|e| BroadcastError(e.0.room))
    }

    /// Subscribes to the lock-free broadcast event stream.
    #[must_use]
    pub fn subscribe_broadcast(&self) -> tokio::sync::broadcast::Receiver<RealtimeBroadcast> {
        self.broadcast_tx.subscribe()
    }

    /// Subscribes a session to a room (e.g. `doc:Sales Invoice:INV-2026-0001`).
    pub fn subscribe(&self, session_id: &str, room: &str) {
        let s_id = CompactString::from(session_id);
        let r_name = CompactString::from(room);

        if let Ok(mut rooms) = self.rooms.write() {
            rooms
                .entry(r_name.clone())
                .or_default()
                .insert(s_id.clone());
        }

        if let Ok(mut subs) = self.subscriptions.write() {
            subs.entry(s_id).or_default().insert(r_name);
        }
    }

    /// Unsubscribes a session from a room.
    pub fn unsubscribe(&self, session_id: &str, room: &str) {
        if let Ok(mut rooms) = self.rooms.write()
            && let Some(sessions) = rooms.get_mut(room)
        {
            sessions.remove(session_id);
        }
        if let Ok(mut subs) = self.subscriptions.write()
            && let Some(rooms) = subs.get_mut(session_id)
        {
            rooms.remove(room);
        }
    }

    /// Cleans up all subscriptions when a session disconnects.
    pub fn remove_session(&self, session_id: &str) {
        if let Ok(mut subs) = self.subscriptions.write()
            && let Some(rooms) = subs.remove(session_id)
            && let Ok(mut rooms_map) = self.rooms.write()
        {
            for r in rooms {
                if let Some(sessions) = rooms_map.get_mut(&r) {
                    sessions.remove(session_id);
                }
            }
        }
    }

    /// Returns list of active session IDs subscribed to a room.
    #[must_use]
    pub fn get_subscribers(&self, room: &str) -> Vec<CompactString> {
        self.rooms
            .read()
            .ok()
            .and_then(|r| r.get(room).map(|s| s.iter().cloned().collect()))
            .unwrap_or_default()
    }
}

/// WebSocket Live Query Streaming & Room Subscription Actor (Milestone 1.8 & 4.3).
pub async fn live_ws_handler(
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let (res, mut session, mut stream) = actix_ws::handle(&req, body)?;
    let hub = req
        .app_data::<web::Data<RealtimeHub>>()
        .map(|h| h.get_ref().clone())
        .unwrap_or_default();
    let mut broadcast_rx = hub.subscribe_broadcast();

    actix_web::rt::spawn(async move {
        let mut subscribed_rooms = HashSet::<CompactString>::new();
        // Default wildcard subscription
        subscribed_rooms.insert("*".into());

        loop {
            tokio::select! {
                // Incoming message from client
                Some(msg_res) = stream.next() => {
                    match msg_res {
                        Ok(actix_ws::Message::Text(text)) => {
                            let text_str = text.trim();
                            if let Some(room) = text_str.strip_prefix("SUBSCRIBE:") {
                                subscribed_rooms.insert(room.trim().into());
                                let _ = session.text(format!("ACK_SUB:{}", room.trim())).await;
                            } else if let Some(room) = text_str.strip_prefix("UNSUBSCRIBE:") {
                                subscribed_rooms.remove(room.trim());
                                let _ = session.text(format!("ACK_UNSUB:{}", room.trim())).await;
                            } else {
                                let ack_frame = format!("LIVE_EVENT:{}", text);
                                let _ = session.text(ack_frame).await;
                            }
                        }
                        Ok(actix_ws::Message::Ping(bytes)) => {
                            let _ = session.pong(&bytes).await;
                        }
                        Ok(actix_ws::Message::Close(reason)) => {
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => {}
                    }
                }
                // Incoming broadcast event from server
                Ok(broadcast) = broadcast_rx.recv() => {
                    if (subscribed_rooms.contains("*") || subscribed_rooms.contains(&broadcast.room))
                        && let Ok(json) = serde_json::to_string(&broadcast.event)
                    {
                        let _ = session.text(format!("EVENT:{}:{}", broadcast.room, json)).await;
                    }
                }
                else => break,
            }
        }
    });

    Ok(res)
}

/// Generates a SurrealQL LIVE SELECT registration query.
#[must_use]
pub fn live_query(tenant_ns: &str, table: &str, id: &str) -> String {
    format!(
        "LIVE SELECT * FROM {}:{} WHERE $tenant = '{}';",
        table, id, tenant_ns
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_query_format() {
        let q = live_query("tenant_acme", "sales_invoice", "INV-001");
        assert!(q.starts_with("LIVE SELECT * FROM sales_invoice:INV-001"));
    }

    #[test]
    fn test_realtime_hub_subscriptions() {
        let hub = RealtimeHub::new();
        let room = "doc:Sales Invoice:INV-2026-0001";

        hub.subscribe("sess-1", room);
        hub.subscribe("sess-2", room);

        let subs = hub.get_subscribers(room);
        assert_eq!(subs.len(), 2);
        assert!(subs.iter().any(|s| s == "sess-1"));
        assert!(subs.iter().any(|s| s == "sess-2"));

        hub.unsubscribe("sess-1", room);
        let subs_after = hub.get_subscribers(room);
        assert_eq!(subs_after.len(), 1);
        assert_eq!(subs_after[0].as_str(), "sess-2");

        hub.remove_session("sess-2");
        assert!(hub.get_subscribers(room).is_empty());
    }

    #[test]
    fn test_realtime_event_serialization() {
        let event = RealtimeEvent::DocUpdate {
            doctype: "Sales Invoice".into(),
            name: "INV-2026-0001".into(),
            modified: "2026-10-17T10:00:00Z".into(),
            user: "cashier@example.com".into(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("doc_update"));
        assert!(json.contains("INV-2026-0001"));
    }

    #[tokio::test]
    async fn test_realtime_hub_broadcast_dispatch() {
        let hub = RealtimeHub::new();
        let mut rx = hub.subscribe_broadcast();

        let event = RealtimeEvent::TaskProgress {
            task_id: "task-001".into(),
            progress: 75,
            description: "Synchronizing GL Ledger".into(),
        };

        let send_res = hub.dispatch("task:task-001", event.clone());
        assert!(send_res.is_ok());

        let received = rx.recv().await.unwrap();
        assert_eq!(received.room.as_str(), "task:task-001");
        assert_eq!(received.event, event);
    }
}
