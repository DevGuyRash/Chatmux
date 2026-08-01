//! Browser-manifest rendering, least-privilege checks, and parity validation.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::config::{Browser, PROVIDER_ORIGINS};

pub(crate) fn render(repo: &Path, browser: Browser, dist_dir: &Path, version: &str) -> Result<()> {
    let source = repo
        .join("extension-src")
        .join(browser.as_str())
        .join("manifest.json");
    let raw = fs::read_to_string(&source)
        .with_context(|| format!("reading manifest source {}", source.display()))?;
    let rendered = raw.replace("__VERSION__", version);
    if rendered.contains("__VERSION__") {
        bail!(
            "rendering {} left an unresolved version placeholder; fix the manifest template",
            source.display()
        );
    }
    let mut manifest: Value = serde_json::from_str(&rendered)
        .with_context(|| format!("parsing manifest source {}", source.display()))?;
    manifest["version_name"] = Value::String(version.to_owned());
    validate_manifest_value(&manifest, browser, version)?;
    let destination = dist_dir.join("manifest.json");
    fs::write(&destination, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("writing staged manifest {}", destination.display()))?;
    Ok(())
}

pub(crate) fn validate(path: &Path, browser: Browser, version: &str) -> Result<()> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("reading staged manifest {}", path.display()))?;
    let manifest: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing staged manifest {}", path.display()))?;
    validate_manifest_value(&manifest, browser, version)
}

pub(crate) fn validate_pair(chrome_path: &Path, firefox_path: &Path) -> Result<()> {
    let chrome: Value = serde_json::from_str(
        &fs::read_to_string(chrome_path)
            .with_context(|| format!("reading Chrome manifest {}", chrome_path.display()))?,
    )?;
    let firefox: Value = serde_json::from_str(
        &fs::read_to_string(firefox_path)
            .with_context(|| format!("reading Firefox manifest {}", firefox_path.display()))?,
    )?;
    for pointer in [
        "/name",
        "/description",
        "/version",
        "/version_name",
        "/optional_host_permissions",
        "/commands",
        "/content_scripts",
        "/web_accessible_resources",
        "/content_security_policy",
    ] {
        if chrome.pointer(pointer) != firefox.pointer(pointer) {
            bail!(
                "Chrome and Firefox manifests diverge at {pointer}; keep shared extension behavior equivalent"
            );
        }
    }
    Ok(())
}

fn validate_manifest_value(manifest: &Value, browser: Browser, version: &str) -> Result<()> {
    let actual_version = manifest
        .get("version")
        .and_then(Value::as_str)
        .context("manifest version is missing or not a string")?;
    if actual_version != version {
        bail!("manifest version {actual_version} does not match workspace version {version}");
    }

    let permissions = string_set(manifest.get("permissions"), "permissions")?;
    if permissions.contains("tabs") {
        bail!("manifest requests the broad tabs permission; use optional provider origins instead");
    }
    let required_permissions: &[&str] = match browser {
        Browser::Chrome => &["storage", "unlimitedStorage", "scripting", "sidePanel"],
        Browser::Firefox => &["storage", "unlimitedStorage", "scripting", "menus"],
    };
    for required in required_permissions {
        if !permissions.contains(*required) {
            bail!(
                "{} manifest is missing required permission {required}",
                browser.as_str()
            );
        }
    }

    if let Some(host_permissions) = manifest.get("host_permissions") {
        let hosts = string_set(Some(host_permissions), "host_permissions")?;
        if !hosts.is_empty() {
            bail!(
                "{} manifest grants provider hosts at install time; move them to optional_host_permissions",
                browser.as_str()
            );
        }
    }
    let optional_hosts = string_set(
        manifest.get("optional_host_permissions"),
        "optional_host_permissions",
    )?;
    let expected_hosts = PROVIDER_ORIGINS
        .iter()
        .map(|origin| (*origin).to_owned())
        .collect::<BTreeSet<_>>();
    if optional_hosts != expected_hosts {
        bail!(
            "{} optional host permissions do not match the supported provider origins",
            browser.as_str()
        );
    }

    validate_content_script_origins(manifest, &expected_hosts)?;
    match browser {
        Browser::Chrome => validate_chrome_shape(manifest)?,
        Browser::Firefox => validate_firefox_shape(manifest)?,
    }
    Ok(())
}

fn validate_content_script_origins(
    manifest: &Value,
    allowed_hosts: &BTreeSet<String>,
) -> Result<()> {
    let scripts = manifest
        .get("content_scripts")
        .and_then(Value::as_array)
        .context("manifest content_scripts is missing or not an array")?;
    for script in scripts {
        let matches = string_set(script.get("matches"), "content_scripts.matches")?;
        if matches.iter().any(|matched| {
            !allowed_hosts
                .iter()
                .any(|allowed| pattern_covers(allowed, matched))
        }) {
            bail!(
                "content script matches an origin outside optional_host_permissions; keep provider access scoped"
            );
        }
    }
    Ok(())
}

fn pattern_covers(allowed: &str, matched: &str) -> bool {
    if allowed == matched {
        return true;
    }
    allowed
        .strip_suffix('*')
        .is_some_and(|prefix| matched.starts_with(prefix))
}

fn validate_chrome_shape(manifest: &Value) -> Result<()> {
    if manifest
        .pointer("/background/service_worker")
        .and_then(Value::as_str)
        != Some("background.js")
        || manifest.pointer("/background/type").and_then(Value::as_str) != Some("module")
    {
        bail!("Chrome manifest must use background.js as a module service worker");
    }
    if manifest
        .pointer("/side_panel/default_path")
        .and_then(Value::as_str)
        != Some("ui/index.html")
    {
        bail!("Chrome side panel must load ui/index.html");
    }
    Ok(())
}

fn validate_firefox_shape(manifest: &Value) -> Result<()> {
    let background_scripts = string_set(
        manifest.pointer("/background/scripts"),
        "background.scripts",
    )?;
    if background_scripts != BTreeSet::from([String::from("background.js")]) {
        bail!("Firefox manifest must use background.js as its background script");
    }
    if manifest
        .pointer("/browser_specific_settings/gecko/data_collection_permissions/required")
        .and_then(Value::as_array)
        .is_none_or(|required| required.as_slice() != [Value::String(String::from("none"))])
    {
        bail!("Firefox manifest must declare required data collection permission none");
    }
    if manifest
        .pointer("/sidebar_action/default_panel")
        .and_then(Value::as_str)
        != Some("ui/index.html")
    {
        bail!("Firefox sidebar must load ui/index.html");
    }
    Ok(())
}

fn string_set(value: Option<&Value>, label: &str) -> Result<BTreeSet<String>> {
    let entries = value
        .and_then(Value::as_array)
        .with_context(|| format!("manifest {label} is missing or not an array"))?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .with_context(|| format!("manifest {label} contains a non-string value"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{render, validate_manifest_value, validate_pair};
    use crate::config::Browser;
    use serde_json::json;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn broad_tabs_permission_is_rejected() {
        let manifest = json!({
            "version": "0.1.0",
            "permissions": ["storage", "unlimitedStorage", "scripting", "sidePanel", "tabs"],
            "optional_host_permissions": []
        });
        let error = validate_manifest_value(&manifest, Browser::Chrome, "0.1.0")
            .expect_err("tabs must be rejected");
        assert!(error.to_string().contains("broad tabs permission"));
    }

    #[test]
    fn source_manifests_are_least_privilege_and_equivalent() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("chatmux-manifests-{nonce}"));
        let chrome = root.join("chrome");
        let firefox = root.join("firefox");
        fs::create_dir_all(&chrome).expect("create Chrome fixture");
        fs::create_dir_all(&firefox).expect("create Firefox fixture");
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask should have a repository parent");
        render(repo, Browser::Chrome, &chrome, "0.1.0").expect("render Chrome manifest");
        render(repo, Browser::Firefox, &firefox, "0.1.0").expect("render Firefox manifest");
        validate_pair(
            &chrome.join("manifest.json"),
            &firefox.join("manifest.json"),
        )
        .expect("browser manifests should preserve shared behavior");
        fs::remove_dir_all(root).expect("remove manifest fixtures");
    }
}
