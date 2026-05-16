const CORE_MANIFESTS: &[&str] = &[
    "crates/ibkr-approval/Cargo.toml",
    "crates/ibkr-audit/Cargo.toml",
    "crates/ibkr-auth/Cargo.toml",
    "crates/ibkr-backend/Cargo.toml",
    "crates/ibkr-config/Cargo.toml",
    "crates/ibkr-cpapi/Cargo.toml",
    "crates/ibkr-domain/Cargo.toml",
    "crates/ibkr-mcp/Cargo.toml",
    "crates/ibkr-oauth/Cargo.toml",
    "crates/ibkr-orders/Cargo.toml",
    "crates/ibkr-risk/Cargo.toml",
    "crates/ibkr-sidecar/Cargo.toml",
];

const PROVIDER_SDK_DEPENDENCIES: &[&str] = &[
    "anthropic",
    "anthropic-ai-sdk",
    "anthropic-sdk",
    "async-openai",
    "openai",
    "openai-api",
    "openai-api-rs",
    "openai_dive",
];

#[test]
fn core_crates_do_not_depend_on_provider_sdks() -> Result<(), Box<dyn std::error::Error>> {
    for manifest_path in CORE_MANIFESTS {
        let manifest = std::fs::read_to_string(manifest_path)?;

        for dependency in PROVIDER_SDK_DEPENDENCIES {
            assert!(
                !declares_dependency(&manifest, dependency),
                "{manifest_path} must not depend on provider SDK {dependency}"
            );
        }
    }

    Ok(())
}

fn declares_dependency(manifest: &str, dependency: &str) -> bool {
    manifest.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#')
            && (trimmed.starts_with(&format!("{dependency} ="))
                || trimmed.starts_with(&format!("{dependency}.workspace"))
                || trimmed.starts_with(&format!("[dependencies.{dependency}]"))
                || trimmed.starts_with(&format!("[dev-dependencies.{dependency}]")))
    })
}
