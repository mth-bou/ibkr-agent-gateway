use ibkr_backend::require_account_id;

#[test]
fn missing_account_context_refuses() {
    let result = require_account_id(None);
    assert!(result.is_err());
}
