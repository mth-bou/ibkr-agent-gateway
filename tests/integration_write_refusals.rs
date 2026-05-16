use ibkr_domain::ErrorCode;

#[test]
fn order_write_commands_are_refused() -> Result<(), Box<dyn std::error::Error>> {
    let error = ibkr_agent_gateway::cli::commands::orders::refuse_write("submit");
    match error {
        Err(error) => {
            assert_eq!(error.code, ErrorCode::ReadonlyWriteForbidden);
            Ok(())
        }
        Ok(()) => Err("write command should refuse".into()),
    }
}
