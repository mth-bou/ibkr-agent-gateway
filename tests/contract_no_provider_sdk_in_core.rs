const CORE_MANIFESTS: &[&str] = &["Cargo.toml"];

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
