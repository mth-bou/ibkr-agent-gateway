use ibkr_agent_gateway::mcp;

fn main() {
    println!("{}", mcp::serve_stdio_description());
}
