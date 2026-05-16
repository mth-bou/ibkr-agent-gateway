use ibkr_agent_gateway::testing::observability::{MetricEvent, StructuredLogEvent};
use std::collections::BTreeMap;

#[test]
fn structured_logs_redact_tokens_cookies_and_local_paths() {
    let fields = BTreeMap::from([
        ("authorization".to_string(), "Bearer raw-token".to_string()),
        ("cookie".to_string(), "ibkr=session".to_string()),
        (
            "config_path".to_string(),
            "/home/user/.ibkr/config".to_string(),
        ),
        ("tool_name".to_string(), "ibkr_health".to_string()),
    ]);

    let event = StructuredLogEvent::new("broker_request", fields);

    assert_eq!(event.fields["authorization"], "[REDACTED]");
    assert_eq!(event.fields["cookie"], "[REDACTED]");
    assert_eq!(event.fields["config_path"], "[REDACTED]");
    assert_eq!(event.fields["tool_name"], "ibkr_health");
}

#[test]
fn metric_labels_reject_account_ids_and_tokens() {
    let labels = BTreeMap::from([
        ("tool".to_string(), "ibkr_orders_list".to_string()),
        ("account".to_string(), "DU1234567".to_string()),
    ]);

    assert!(MetricEvent::new("ibkr_tool_latency_ms", labels, 42.0).is_err());

    let labels = BTreeMap::from([
        ("tool".to_string(), "ibkr_orders_list".to_string()),
        ("result".to_string(), "refused".to_string()),
    ]);
    assert!(MetricEvent::new("ibkr_tool_latency_ms", labels, 42.0).is_ok());
}
