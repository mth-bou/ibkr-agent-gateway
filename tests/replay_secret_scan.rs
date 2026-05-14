use std::path::Path;

#[test]
fn fixture_outputs_do_not_contain_secret_markers() -> Result<(), Box<dyn std::error::Error>> {
    let forbidden_markers = [
        "authorization",
        "bearer ",
        "set-cookie",
        "cookie:",
        "password",
        "credential",
        "access_token",
        "refresh_token",
        "private_key",
        "client_secret",
    ];

    for path in fixture_paths(Path::new("tests/fixtures/cpapi"))? {
        let raw = std::fs::read_to_string(&path)?;
        let lowered = raw.to_ascii_lowercase();
        for marker in forbidden_markers {
            assert!(
                !lowered.contains(marker),
                "fixture contains forbidden marker {marker}: {}",
                path.display()
            );
        }
    }

    Ok(())
}

fn fixture_paths(root: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}
