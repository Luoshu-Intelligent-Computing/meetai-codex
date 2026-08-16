use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use async_channel::Sender;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::McpToolCallProgressEvent;
use codex_rmcp_client::SendProgress;
use rmcp::model::ProgressNotificationParam;

#[derive(Clone)]
struct ProgressRoute {
    event_id: String,
    call_id: String,
}

#[derive(Default)]
struct ProgressRoutes {
    by_token: HashMap<String, ProgressRoute>,
    bound_tokens: HashMap<String, String>,
    token_owners: HashMap<String, String>,
}

/// Bridges generic MCP progress notifications to the Codex event channel.
/// Routes are registered only for the duration of an active MCP call; unknown
/// and late notifications are intentionally dropped.
#[derive(Clone)]
pub(crate) struct McpProgressRouter {
    tx_event: Option<Sender<Event>>,
    routes: Arc<Mutex<ProgressRoutes>>,
}

impl McpProgressRouter {
    pub(crate) fn new(tx_event: Option<Sender<Event>>) -> Self {
        Self {
            tx_event,
            routes: Arc::new(Mutex::new(ProgressRoutes::default())),
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
        routes.by_token.insert(
            token.to_string(),
            ProgressRoute {
                event_id: event_id.to_string(),
                call_id: call_id.to_string(),
            },
        );
    }

    pub(crate) fn bind(&self, token: &str, generated_token: &str) {
        let Ok(mut routes) = self.routes.lock() else {
            return;
        };
        let Some(route) = routes.by_token.get(token).cloned() else {
            return;
        };
        if let Some(previous_generated_token) = routes.bound_tokens.remove(token)
            && routes
                .token_owners
                .get(&previous_generated_token)
                .is_some_and(|owner| owner == token)
        {
            routes.token_owners.remove(&previous_generated_token);
            routes.by_token.remove(&previous_generated_token);
        }
        if let Some(previous_owner) = routes
            .token_owners
            .insert(generated_token.to_string(), token.to_string())
        {
            routes.bound_tokens.remove(&previous_owner);
        }
        routes.by_token.insert(generated_token.to_string(), route);
        routes
            .bound_tokens
            .insert(token.to_string(), generated_token.to_string());
    }

    pub(crate) fn unregister(&self, token: &str) {
        if let Ok(mut routes) = self.routes.lock() {
            routes.by_token.remove(token);
            if let Some(generated_token) = routes.bound_tokens.remove(token)
                && routes
                    .token_owners
                    .get(&generated_token)
                    .is_some_and(|owner| owner == token)
            {
                routes.token_owners.remove(&generated_token);
                routes.by_token.remove(&generated_token);
            }
        }
    }

    fn publish(&self, params: ProgressNotificationParam) {
        let token = params.progress_token.0.to_string();
        let route = self
            .routes
            .lock()
            .ok()
            .and_then(|routes| routes.by_token.get(&token).cloned());
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
    use rmcp::model::NumberOrString;
    use rmcp::model::ProgressToken;

    #[test]
    fn routes_progress_to_registered_call_and_drops_late_notifications() {
        let (tx, rx) = async_channel::bounded(2);
        let router = McpProgressRouter::new(Some(tx));
        router.register("call-1", "turn-1", "call-1");
        router.bind("call-1", "0");
        let callback = router.callback().expect("callback should be enabled");
        callback(
            ProgressNotificationParam::new(ProgressToken(NumberOrString::Number(0)), 1.0)
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

        router.unregister("call-1");
        callback(ProgressNotificationParam::new(
            ProgressToken(NumberOrString::Number(0)),
            2.0,
        ));
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn keeps_concurrent_and_rebound_progress_routes_isolated() {
        let (tx, rx) = async_channel::bounded(4);
        let router = McpProgressRouter::new(Some(tx));
        router.register("call-1", "turn-1", "call-1");
        router.register("call-2", "turn-2", "call-2");
        router.bind("call-1", "0");
        router.bind("call-2", "1");
        router.bind("call-1", "2");
        let callback = router.callback().expect("callback should be enabled");

        callback(ProgressNotificationParam::new(
            ProgressToken(NumberOrString::Number(0)),
            1.0,
        ));
        callback(ProgressNotificationParam::new(
            ProgressToken(NumberOrString::Number(1)),
            2.0,
        ));
        callback(ProgressNotificationParam::new(
            ProgressToken(NumberOrString::Number(2)),
            3.0,
        ));

        let second = rx
            .try_recv()
            .expect("second call progress should be queued");
        let first = rx
            .try_recv()
            .expect("rebound first call progress should be queued");
        assert!(rx.try_recv().is_err());
        assert!(matches!(
            second.msg,
            EventMsg::McpToolCallProgress(McpToolCallProgressEvent { call_id, .. })
                if call_id == "call-2"
        ));
        assert!(matches!(
            first.msg,
            EventMsg::McpToolCallProgress(McpToolCallProgressEvent { call_id, .. })
                if call_id == "call-1"
        ));

        router.unregister("call-1");
        callback(ProgressNotificationParam::new(
            ProgressToken(NumberOrString::Number(1)),
            4.0,
        ));
        assert!(rx.try_recv().is_ok());
    }
}
