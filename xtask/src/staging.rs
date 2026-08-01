//! Strict staging of extension sources and generated runtime artifacts.

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{Browser, WASM_CRATES};

pub(crate) fn recreate_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("removing stale staged directory {}", path.display()))?;
    }
    fs::create_dir_all(path)
        .with_context(|| format!("creating staged directory {}", path.display()))?;
    Ok(())
}

pub(crate) fn stage_extension_assets(repo: &Path, browser: Browser, dist_dir: &Path) -> Result<()> {
    let shell_background = repo
        .join("extension-src")
        .join(browser.as_str())
        .join("background.js");
    if shell_background.exists() {
        bail!(
            "{} shadows the shared background runtime; keep one audited runtime in extension-src/common/background.js",
            shell_background.display()
        );
    }
    copy_tree(&repo.join("extension-src/common"), dist_dir, |_| true)?;
    copy_tree(
        &repo.join("extension-src").join(browser.as_str()),
        dist_dir,
        |path| path.file_name().is_none_or(|name| name != "manifest.json"),
    )?;
    let license = repo.join("LICENSE");
    if !license.is_file() {
        bail!("root LICENSE is missing; add the MIT license before packaging");
    }
    fs::copy(&license, dist_dir.join("LICENSE"))
        .with_context(|| format!("copying {}", license.display()))?;
    Ok(())
}

pub(crate) fn stage_ui(repo: &Path, dist_dir: &Path) -> Result<()> {
    let source = repo.join("chatmux-ui/dist");
    if !source.join("index.html").is_file() {
        bail!("UI build output is missing; install pinned trunk and rebuild the extension");
    }
    let target = dist_dir.join("ui");
    copy_tree(&source, &target, |_| true)?;
    rewrite_staged_ui_index(&target)
}

pub(crate) fn stage_wasm(repo: &Path, dist_dir: &Path) -> Result<()> {
    let target = dist_dir.join("wasm");
    fs::create_dir_all(&target)
        .with_context(|| format!("creating Wasm stage {}", target.display()))?;
    for crate_name in WASM_CRATES {
        let source = repo.join(crate_name).join("pkg");
        if !source.is_dir() {
            bail!(
                "Wasm output for {crate_name} is missing; install pinned wasm-pack and rebuild the extension"
            );
        }
        copy_tree(&source, &target, |path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("js" | "wasm")
            )
        })?;
    }
    Ok(())
}

pub(crate) fn validate_required_files(dist_dir: &Path) -> Result<()> {
    let required = [
        "LICENSE",
        "manifest.json",
        "background.js",
        "completion-stability.mjs",
        "send-readiness.mjs",
        "content-gpt.js",
        "content-gemini.js",
        "content-grok.js",
        "content-claude.js",
        "ui/index.html",
        "ui/bootstrap.js",
        // Declared by both manifests and by the UI favicon. A missing icon
        // does not fail the browser's manifest parse — it silently falls back
        // to the generic puzzle-piece — so packaging has to catch it here.
        "icons/icon-16.png",
        "icons/icon-32.png",
        "icons/icon-48.png",
        "icons/icon-96.png",
        "icons/icon-128.png",
        "wasm/chatmux_core.js",
        "wasm/chatmux_core_bg.wasm",
        "wasm/chatmux_adapter_gpt.js",
        "wasm/chatmux_adapter_gpt_bg.wasm",
        "wasm/chatmux_adapter_gemini.js",
        "wasm/chatmux_adapter_gemini_bg.wasm",
        "wasm/chatmux_adapter_grok.js",
        "wasm/chatmux_adapter_grok_bg.wasm",
        "wasm/chatmux_adapter_claude.js",
        "wasm/chatmux_adapter_claude_bg.wasm",
    ];
    for relative in required {
        if !dist_dir.join(relative).is_file() {
            bail!("staged extension is missing {relative}; rebuild all UI and Wasm artifacts");
        }
    }
    require_one_hashed_asset(dist_dir, "ui", "chatmux-ui-", ".js")?;
    require_one_hashed_asset(dist_dir, "ui", "chatmux-ui-", "_bg.wasm")?;
    Ok(())
}

fn require_one_hashed_asset(
    dist_dir: &Path,
    relative_dir: &str,
    prefix: &str,
    suffix: &str,
) -> Result<()> {
    let directory = dist_dir.join(relative_dir);
    let count = fs::read_dir(&directory)
        .with_context(|| format!("reading staged asset directory {}", directory.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .count();
    if count != 1 {
        bail!(
            "staged {relative_dir} must contain exactly one {prefix}*{suffix} asset, found {count}"
        );
    }
    Ok(())
}

fn copy_tree(source: &Path, target: &Path, include: impl Fn(&Path) -> bool) -> Result<()> {
    if !source.is_dir() {
        bail!("required build input {} is missing", source.display());
    }
    let mut files = Vec::<PathBuf>::new();
    for entry in WalkDir::new(source) {
        let entry = entry.with_context(|| format!("walking {}", source.display()))?;
        if entry.file_type().is_file() && include(entry.path()) {
            files.push(entry.into_path());
        }
    }
    files.sort_by_key(|path| path.to_string_lossy().replace('\\', "/"));
    for path in files {
        let relative = path
            .strip_prefix(source)
            .with_context(|| format!("copy input {} escaped its source root", path.display()))?;
        let destination = target.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        fs::copy(&path, &destination).with_context(|| format!("copying {}", path.display()))?;
    }
    Ok(())
}

fn rewrite_staged_ui_index(ui_target: &Path) -> Result<()> {
    let index_path = ui_target.join("index.html");
    let html = fs::read_to_string(&index_path)
        .with_context(|| format!("reading staged UI {}", index_path.display()))?;
    let rewritten = rewrite_root_absolute_ui_paths(&html);
    let rewritten = externalize_inline_module_bootstrap(ui_target, &rewritten)?;
    fs::write(&index_path, rewritten)
        .with_context(|| format!("writing staged UI {}", index_path.display()))?;
    Ok(())
}

fn rewrite_root_absolute_ui_paths(html: &str) -> String {
    html.replace("href=\"/", "href=\"./")
        .replace("src=\"/", "src=\"./")
        .replace("from '/", "from './")
        .replace("from \"/", "from \"./")
        .replace(": '/", ": './")
        .replace(": \"/", ": \"./")
}

fn externalize_inline_module_bootstrap(ui_target: &Path, html: &str) -> Result<String> {
    const OPEN: &str = "<script type=\"module\">";
    const CLOSE: &str = "</script>";
    const TAG: &str = "<script type=\"module\" src=\"./bootstrap.js\"></script>";
    const MARKERS: [&str; 2] = ["TrunkApplicationStarted", "module_or_path:"];

    let mut search_from = 0usize;
    while let Some(start_offset) = html[search_from..].find(OPEN) {
        let start = search_from + start_offset;
        let body_start = start + OPEN.len();
        let end_offset = html[body_start..]
            .find(CLOSE)
            .context("staged UI index contains an unterminated inline module script")?;
        let end = body_start + end_offset;
        let body = html[body_start..end].trim();
        let range_end = end + CLOSE.len();
        if MARKERS.iter().all(|marker| body.contains(marker)) {
            fs::write(ui_target.join("bootstrap.js"), format!("{body}\n"))?;
            return Ok(format!("{}{}{}", &html[..start], TAG, &html[range_end..]));
        }
        search_from = range_end;
    }
    bail!("staged UI index did not contain a recognizable Trunk bootstrap module script")
}

#[cfg(test)]
mod tests {
    use super::rewrite_root_absolute_ui_paths;

    #[test]
    fn root_absolute_ui_paths_become_extension_relative() {
        let html = r#"<link href="/tokens.css"><script type="module">import init from '/ui.js'; const wasm = await init({ module_or_path: "/ui_bg.wasm" });</script>"#;
        let rewritten = rewrite_root_absolute_ui_paths(html);
        assert!(rewritten.contains("href=\"./tokens.css\""));
        assert!(rewritten.contains("from './ui.js'"));
        assert!(rewritten.contains("module_or_path: \"./ui_bg.wasm\""));
    }
}
