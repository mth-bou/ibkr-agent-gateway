use ibkr_agent_gateway::prelude::*;

#[tokio::main]
async fn main() -> Result<(), GatewayError> {
    let gateway = Gateway::new(GatewayConfig::fake_local())?;
    let session = gateway.session_status().await?;
    let accounts = gateway.list_accounts().await?;

    println!(
        "session={:?} visible_accounts={}",
        session.status,
        accounts.len()
    );

    Ok(())
}
