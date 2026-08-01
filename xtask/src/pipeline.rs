//! Deterministic extension build, staging, packaging, and verification pipeline.

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Browser;
use crate::config::WASM_CRATES;
use crate::{archive, fingerprint, manifest, metadata, staging, tools};

/// Coordinates one source snapshot through build, staging, and packaging.
pub(crate) struct Pipeline {
    repo: PathBuf,
}

impl Pipeline {
    /// Creates a pipeline rooted at the repository under qualification.
    pub(crate) fn new(repo: &Path) -> Self {
        Self {
            repo: repo.to_path_buf(),
        }
    }

    /// Builds all shared UI and Wasm artifacts exactly once.
    pub(crate) fn build_artifacts(&self) -> Result<String> {
        let tool_paths = tools::require_packaging_tools()?;
        let source_before = fingerprint::source_fingerprint(&self.repo)?;
        let ui_dir = self.repo.join("chatmux-ui");
        tools::run(
            Command::new(&tool_paths.trunk)
                .arg("build")
                .arg("--release")
                .current_dir(&ui_dir)
                .env_remove("NO_COLOR")
                .env_remove("CLICOLOR")
                .env_remove("CLICOLOR_FORCE"),
            "trunk build --release",
        )?;

        for crate_name in WASM_CRATES {
            tools::run(
                Command::new(&tool_paths.wasm_pack)
                    .arg("build")
                    .arg("--target")
                    .arg("web")
                    .arg("--release")
                    .arg("--no-typescript")
                    .arg("--no-pack")
                    .current_dir(self.repo.join(crate_name)),
                &format!("wasm-pack build for {crate_name}"),
            )?;
        }

        let source_after = fingerprint::source_fingerprint(&self.repo)?;
        if source_before != source_after {
            bail!(
                "extension sources changed while artifacts were building; rerun the build from a stable source tree"
            );
        }
        Ok(source_before)
    }

    /// Stages and verifies one unpacked browser extension.
    pub(crate) fn stage(&self, browser: Browser, source_fingerprint: &str) -> Result<PathBuf> {
        let current_source = fingerprint::source_fingerprint(&self.repo)?;
        if current_source != source_fingerprint {
            bail!(
                "extension sources changed after the shared build; rebuild before staging {}",
                browser.as_str()
            );
        }
        let version = self.workspace_version()?;
        let dist_dir = self.dist_dir(browser);
        staging::recreate_dir(&dist_dir)?;
        staging::stage_extension_assets(&self.repo, browser, &dist_dir)?;
        manifest::render(&self.repo, browser, &dist_dir, &version)?;
        staging::stage_ui(&self.repo, &dist_dir)?;
        staging::stage_wasm(&self.repo, &dist_dir)?;
        staging::validate_required_files(&dist_dir)?;
        metadata::write(&dist_dir, browser, &version, source_fingerprint)?;
        self.verify_dist(browser)?;
        Ok(dist_dir)
    }

    /// Restages already-built UI and Wasm artifacts for a fast local test loop.
    ///
    /// Final qualification still uses `dist`, which rebuilds every artifact
    /// from one source snapshot before staging it.
    pub(crate) fn stage_existing_artifacts(&self, browser: Browser) -> Result<PathBuf> {
        let source_fingerprint = fingerprint::source_fingerprint(&self.repo)?;
        self.stage(browser, &source_fingerprint)
    }

    /// Packages and verifies one browser archive from an already-built snapshot.
    pub(crate) fn package(&self, browser: Browser, source_fingerprint: &str) -> Result<PathBuf> {
        let dist_dir = self.stage(browser, source_fingerprint)?;
        let archive_path = self.archive_path(browser)?;
        archive::create(&dist_dir, &archive_path)?;
        archive::validate_matches(&dist_dir, &archive_path)?;
        Ok(archive_path)
    }

    /// Verifies that existing staged and archived artifacts match current sources.
    pub(crate) fn verify_existing(&self, browser: Browser) -> Result<()> {
        self.verify_dist(browser)?;
        let archive_path = self.archive_path(browser)?;
        if !archive_path.is_file() {
            bail!(
                "{} package is missing; run just package-{}",
                archive_path.display(),
                browser.as_str()
            );
        }
        archive::validate_matches(&self.dist_dir(browser), &archive_path)
    }

    /// Verifies an unpacked extension without requiring a ZIP archive.
    pub(crate) fn verify_dist(&self, browser: Browser) -> Result<()> {
        let dist_dir = self.dist_dir(browser);
        if !dist_dir.is_dir() {
            bail!(
                "{} staged extension is missing; run just dist-{}",
                browser.as_str(),
                browser.as_str()
            );
        }
        let version = self.workspace_version()?;
        let source_fingerprint = fingerprint::source_fingerprint(&self.repo)?;
        staging::validate_required_files(&dist_dir)?;
        manifest::validate(&dist_dir.join("manifest.json"), browser, &version)?;
        metadata::validate(&dist_dir, browser, &version, &source_fingerprint)
    }

    /// Verifies shared Chrome/Firefox manifest behavior after both are staged.
    pub(crate) fn validate_browser_parity(&self) -> Result<()> {
        manifest::validate_pair(
            &self.dist_dir(Browser::Chrome).join("manifest.json"),
            &self.dist_dir(Browser::Firefox).join("manifest.json"),
        )
    }

    /// Removes generated extension distributions and archives.
    pub(crate) fn clean(&self) -> Result<()> {
        for relative in ["extension-dist", "extension-packages"] {
            let path = self.repo.join(relative);
            if path.exists() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("removing generated output {}", path.display()))?;
            }
        }
        Ok(())
    }

    fn workspace_version(&self) -> Result<String> {
        let raw = fs::read_to_string(self.repo.join("Cargo.toml"))
            .context("reading root Cargo.toml for extension version")?;
        let value: toml::Value = toml::from_str(&raw)?;
        value
            .get("workspace")
            .and_then(|workspace| workspace.get("package"))
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .context("workspace.package.version is missing from Cargo.toml")
    }

    fn dist_dir(&self, browser: Browser) -> PathBuf {
        self.repo.join("extension-dist").join(browser.as_str())
    }

    fn archive_path(&self, browser: Browser) -> Result<PathBuf> {
        Ok(self.repo.join("extension-packages").join(format!(
            "chatmux-{}-{}.zip",
            browser.as_str(),
            self.workspace_version()?
        )))
    }
}
