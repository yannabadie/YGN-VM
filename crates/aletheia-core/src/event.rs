use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The kind of event being recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ToolUse,
    FileEdit,
    ShellExec,
    PrAction,
    TestRun,
    Custom,
}

/// Context associated with an event (session, repo, branch, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventContext {
    pub session_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl EventContext {
    /// Create a minimal context with only a session ID.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            repo: None,
            branch: None,
            pr_number: None,
            tool: None,
            policy: None,
            result: None,
        }
    }
}

/// Return the current time as Unix epoch milliseconds.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// A single recorded event in an evidence pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    /// Unix epoch milliseconds.
    pub timestamp: u64,
    pub kind: EventKind,
    pub source: String,
    pub context: EventContext,
    pub payload: serde_json::Value,
}

impl Event {
    /// Create a new event with a UUID v7 id and the current UTC timestamp.
    pub fn new(
        kind: EventKind,
        source: impl Into<String>,
        context: EventContext,
        payload: serde_json::Value,
    ) -> Self {
        let id = Uuid::now_v7().to_string();
        let timestamp = now_millis();
        Self {
            id,
            timestamp,
            kind,
            source: source.into(),
            context,
            payload,
        }
    }

    /// Parse a JSONL line into an Event.
    ///
    /// Expected JSON fields: `kind` (optional), `source` (optional), `payload` (required or
    /// falls back to the entire object). Missing `kind` defaults to `Custom`; missing `source`
    /// defaults to `"jsonl"`. Trailing `\r` is stripped for Windows compatibility.
    /// Missing `timestamp` defaults to the current time in milliseconds.
    pub fn from_json_line(line: &str, session_id: impl Into<String>) -> Result<Self, String> {
        // Strip trailing \r (Windows line endings)
        let line = line.trim_end_matches('\r');

        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("JSON parse error: {e}"))?;

        let kind: EventKind = value
            .get("kind")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(EventKind::Custom);

        let source = value
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("jsonl")
            .to_string();

        // Use "payload" field if present, otherwise use the whole object as payload
        let payload = value
            .get("payload")
            .cloned()
            .unwrap_or_else(|| value.clone());

        let timestamp: u64 = value
            .get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(now_millis);

        let context = EventContext::new(session_id);

        Ok(Self {
            id: Uuid::now_v7().to_string(),
            timestamp,
            kind,
            source,
            context,
            payload,
        })
    }

    /// Wrap a plain text string as a Custom event.
    pub fn from_plain_text(text: impl Into<String>, session_id: impl Into<String>) -> Self {
        let text = text.into();
        let context = EventContext::new(session_id);
        let payload = serde_json::json!({ "text": text });
        Self::new(EventKind::Custom, "plain_text", context, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialization_roundtrip() {
        let ctx = EventContext::new("sess-abc");
        let payload = serde_json::json!({"tool": "bash", "cmd": "ls"});
        let event = Event::new(EventKind::ToolUse, "agent", ctx, payload.clone());

        let json = serde_json::to_string(&event).expect("serialize");
        let restored: Event = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(event.id, restored.id);
        assert_eq!(event.kind, restored.kind);
        assert_eq!(event.source, restored.source);
        assert_eq!(event.context.session_id, restored.context.session_id);
        assert_eq!(event.payload, restored.payload);
        assert_eq!(event.timestamp, restored.timestamp);
    }

    #[test]
    fn event_from_json_line_valid() {
        let line = r#"{"kind":"tool_use","source":"bash","payload":{"cmd":"ls"}}"#;
        let event = Event::from_json_line(line, "sess-1").expect("parse");

        assert_eq!(event.kind, EventKind::ToolUse);
        assert_eq!(event.source, "bash");
        assert_eq!(event.context.session_id, "sess-1");
        assert_eq!(event.payload, serde_json::json!({"cmd": "ls"}));
        // timestamp defaults to current time (non-zero)
        assert!(event.timestamp > 0);
    }

    #[test]
    fn event_from_json_line_minimal() {
        // Only payload field, no kind or source
        let line = r#"{"payload":{"msg":"hello"}}"#;
        let event = Event::from_json_line(line, "sess-2").expect("parse");

        assert_eq!(event.kind, EventKind::Custom);
        assert_eq!(event.source, "jsonl");
        assert_eq!(event.payload, serde_json::json!({"msg": "hello"}));
    }

    #[test]
    fn event_from_json_line_with_timestamp() {
        let line = r#"{"payload":{"msg":"hello"},"timestamp":1700000000000}"#;
        let event = Event::from_json_line(line, "sess-ts").expect("parse");

        assert_eq!(event.timestamp, 1700000000000u64);
    }

    #[test]
    fn event_from_plain_text() {
        let event = Event::from_plain_text("hello world", "sess-3");

        assert_eq!(event.kind, EventKind::Custom);
        assert_eq!(event.source, "plain_text");
        assert_eq!(event.context.session_id, "sess-3");
        assert_eq!(event.payload, serde_json::json!({"text": "hello world"}));
    }

    #[test]
    fn event_context_with_all_fields() {
        let ctx = EventContext {
            session_id: "sess-4".to_string(),
            repo: Some("acme/repo".to_string()),
            branch: Some("main".to_string()),
            pr_number: Some(42),
            tool: Some("bash".to_string()),
            policy: Some("strict".to_string()),
            result: Some("ok".to_string()),
        };

        let json = serde_json::to_value(&ctx).expect("serialize");
        assert_eq!(json["session_id"], "sess-4");
        assert_eq!(json["repo"], "acme/repo");
        assert_eq!(json["branch"], "main");
        assert_eq!(json["pr_number"], 42);
        assert_eq!(json["tool"], "bash");
        assert_eq!(json["policy"], "strict");
        assert_eq!(json["result"], "ok");
    }

    #[test]
    fn event_context_skips_none_fields() {
        let ctx = EventContext::new("sess-5");
        let json = serde_json::to_value(&ctx).expect("serialize");

        assert_eq!(json["session_id"], "sess-5");
        // None fields must be absent from the serialized output
        assert!(json.get("repo").is_none());
        assert!(json.get("branch").is_none());
        assert!(json.get("pr_number").is_none());
        assert!(json.get("tool").is_none());
        assert!(json.get("policy").is_none());
        assert!(json.get("result").is_none());
    }
}
