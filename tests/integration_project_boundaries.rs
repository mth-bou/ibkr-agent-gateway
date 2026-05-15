#[test]
fn workspace_does_not_include_later_feature_crates() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = std::fs::read_to_string("Cargo.toml")?;
    let forbidden_members = ["crates/ibkr-live", "crates/ibkr-ops"];

    for member in forbidden_members {
        assert!(!manifest.contains(member));
    }

    Ok(())
}
