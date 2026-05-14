# Data Model: OpenAI, Anthropic, and MCP Client Compatibility

## ProviderTarget

**Fields**: `provider`, `client_kind`, `mcp_transport`, `auth_mode`, `enabled_features`.

## CompatibilityScenario

**Fields**: `scenario_id`, `provider_target`, `tool_name`, `input_fixture`, `expected_shape`, `expected_auth_behavior`.

## CompatibilitySnapshot

**Fields**: `snapshot_id`, `scenario_id`, `tool_schema_hash`, `response_hash`, `redaction_report`, `created_at`.

## RedactionReport

**Fields**: `checked_fields`, `leaks_found`, `redactions`, `secret_scan_result`.
