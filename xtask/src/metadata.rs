//! Reproducible package identity and staged-artifact integrity metadata.

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use crate::config::{BUILD_METADATA_FILE, Browser, TRUNK_VERSION, WASM_PACK_VERSION};
use crate::fingerprint;

pub(crate) fn write(
    dist_dir: &Path,
    browser: Browser,
    version: &str,
    source_fingerprint: &str,
) -> Result<()> {
    let artifact_fingerprint = fingerprint::artifact_fingerprint(dist_dir)?;
    let metadata = json!({
        "schema_version": 1,
        "browser": browser.as_str(),
        "version": version,
        "source_fingerprint": source_fingerprint,
        "artifact_fingerprint": artifact_fingerprint,
        "tools": {
            "trunk": TRUNK_VERSION,
            "wasm_pack": WASM_PACK_VERSION
        }
    });
    let path = dist_dir.join(BUILD_METADATA_FILE);
    fs::write(&path, serde_json::to_string_pretty(&metadata)?)
        .with_context(|| format!("writing build metadata {}", path.display()))?;
    Ok(())
}

pub(crate) fn validate(
    dist_dir: &Path,
    browser: Browser,
    version: &str,
    expected_source_fingerprint: &str,
) -> Result<()> {
    let path = dist_dir.join(BUILD_METADATA_FILE);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("reading build metadata {}", path.display()))?;
    let metadata: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing build metadata {}", path.display()))?;
    require_equal(&metadata, "/schema_version", &json!(1), "schema version")?;
    require_equal(
        &metadata,
        "/browser",
        &json!(browser.as_str()),
        "browser target",
    )?;
    require_equal(&metadata, "/version", &json!(version), "workspace version")?;
    require_equal(
        &metadata,
        "/source_fingerprint",
        &json!(expected_source_fingerprint),
        "source fingerprint",
    )?;
    require_equal(
        &metadata,
        "/tools/trunk",
        &json!(TRUNK_VERSION),
        "trunk version",
    )?;
    require_equal(
        &metadata,
        "/tools/wasm_pack",
        &json!(WASM_PACK_VERSION),
        "wasm-pack version",
    )?;
    let actual_artifact = fingerprint::artifact_fingerprint(dist_dir)?;
    require_equal(
        &metadata,
        "/artifact_fingerprint",
        &json!(actual_artifact),
        "artifact fingerprint",
    )?;
    Ok(())
}

fn require_equal(metadata: &Value, pointer: &str, expected: &Value, label: &str) -> Result<()> {
    let actual = metadata
        .pointer(pointer)
        .with_context(|| format!("build metadata is missing {label}"))?;
    if actual != expected {
        bail!(
            "build metadata {label} is stale or invalid; rebuild the extension before launching or packaging"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate, write};
    use crate::config::Browser;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stale_source_metadata_is_rejected() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("chatmux-metadata-{nonce}"));
        fs::create_dir_all(&root).expect("create metadata fixture");
        fs::write(root.join("manifest.json"), "{}").expect("write fixture artifact");
        write(&root, Browser::Chrome, "0.1.0", "fnv1a64:old").expect("write metadata fixture");

        let error = validate(&root, Browser::Chrome, "0.1.0", "fnv1a64:new")
            .expect_err("stale source must fail");
        fs::remove_dir_all(root).expect("remove metadata fixture");
        assert!(error.to_string().contains("source fingerprint"));
    }
}
