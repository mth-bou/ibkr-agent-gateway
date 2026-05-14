#[test]
fn mcp_order_preview_schema_has_preview_scope_and_required_fields() {
    let schema = ibkr_mcp::tools::order_preview::order_preview_schema();

    assert_eq!(schema.name, "ibkr_order_preview");
    assert_eq!(schema.scope, "ibkr:orders:preview");
    assert_eq!(
        schema.input_schema["required"],
        serde_json::json!([
            "account_id",
            "symbol",
            "side",
            "quantity",
            "order_type",
            "limit_price",
            "time_in_force"
        ])
    );
}
