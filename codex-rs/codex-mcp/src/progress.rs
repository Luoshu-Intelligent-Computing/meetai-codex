use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_channel::Sender;
use codex_protocol::protocol::{Event, EventMsg, McpToolCallProgressEvent};
use codex_rmcp_client::SendProgress;
use rmcp::model::ProgressNotificationParam;

#[derive(Clone)]
struct ProgressRoute {
    event_id: String,
    call_id: String,
}

/// Bridges generic MCP progress notifications to the Codex event channel.
/// Routes are registered only for the duration of an active MCP call; unknown
/// and late notifications are intentionally dropped.
#[derive(Clone)]
pub(crate) struct McpProgressRouter {
    tx_event: Option<Sender<Event>>,
    routes: Arc<Mutex<HashMap<String, ProgressRoute>>>,
}

impl McpProgressRouter {
    pub(crate) fn new(tx_event: Option<Sender<Event>>) -> Self {
        Self {
            tx_event,
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn callback(&self) -> Option<SendProgress> {
        self.tx_event.as_ref()?;
        let router = self.clone();
        Some(Arc::new(move |params| router.publish(params)))
    }

    pub(crate) fn register(&self, token: &str, event_id: &str, call_id: &str) {
        let Ok(mut routes) = self.routes.lock() else {
            return;
        };
        routes.insert(
            token.to_string(),
            ProgressRoute {
                event_id: event_id.to_string(),
                call_id: call_id.to_string(),
            },
        );
    }

    pub(crate) fn unregister(&self, token: &str) {
        if let Ok(mut routes) = self.routes.lock() {
            routes.remove(token);
        }
    }

    fn publish(&self, params: ProgressNotificationParam) {
        let token = params.progress_token.0.to_string();
        let route = self
            .routes
            .lock()
            .ok()
            .and_then(|routes| routes.get(&token).cloned());
        let Some(route) = route else {
            return;
        };
        let Some(tx_event) = self.tx_event.as_ref() else {
            return;
        };
        let _ = tx_event.try_send(Event {
            id: route.event_id,
            msg: EventMsg::McpToolCallProgress(McpToolCallProgressEvent {
                call_id: route.call_id,
                progress: params.progress,
                total: params.total,
                message: params.message,
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{NumberOrString, ProgressToken};

    #[test]
    fn routes_progress_to_registered_call_and_drops_late_notifications() {
        let (tx, rx) = async_channel::bounded(2);
        let router = McpProgressRouter::new(Some(tx));
        router.register("token", "turn-1", "call-1");
        let callback = router.callback().expect("callback should be enabled");
        callback(
            ProgressNotificationParam::new(
                ProgressToken(NumberOrString::String("token".into())),
                1.0,
            )
            .with_message("reading"),
        );

        let event = rx.try_recv().expect("progress event should be queued");
        assert_eq!(event.id, "turn-1");
        assert!(matches!(
            event.msg,
            EventMsg::McpToolCallProgress(McpToolCallProgressEvent {
                call_id,
                progress: 1.0,
                message: Some(message),
                ..
            }) if call_id == "call-1" && message == "reading"
        ));

        router.unregister("token");
        callback(ProgressNotificationParam::new(
            ProgressToken(NumberOrString::String("token".into())),
            2.0,
        ));
        assert!(rx.try_recv().is_err());
    }
}
