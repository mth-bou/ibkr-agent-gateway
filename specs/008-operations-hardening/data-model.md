# Data Model: Operations, Export, Replay, and Hardening

## AuditExport

**Fields**: `export_id`, `format`, `range`, `redaction_policy`, `created_at`, `file_hash`.

## ReplayCase

**Fields**: `case_id`, `input_event`, `expected_decision`, `expected_output_shape`, `secret_scan_expectation`.

## MetricEvent

**Fields**: `metric_name`, `labels`, `value`, `timestamp`.

**Rules**: Labels must not contain account IDs, tokens, cookies, credentials, local paths, or raw broker session material.

## IncidentReview

**Fields**: `incident_id`, `time_range`, `summary`, `affected_tools`, `audit_event_ids`, `action_items`.
