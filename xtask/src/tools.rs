//! Pinned packaging-tool discovery and command execution.

use anyhow::{Context, Result, bail};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{TRUNK_VERSION, WASM_PACK_VERSION};

pub(crate) struct ToolPaths {
    pub(crate) trunk: PathBuf,
    pub(crate) wasm_pack: PathBuf,
}

pub(crate) fn require_packaging_tools() -> Result<ToolPaths> {
    Ok(ToolPaths {
        trunk: require_pinned_tool("trunk", TRUNK_VERSION)?,
        wasm_pack: require_pinned_tool("wasm-pack", WASM_PACK_VERSION)?,
    })
}

pub(crate) fn check_tool_report() -> Result<Vec<String>> {
    let tools = require_packaging_tools()?;
    Ok(vec![
        format!(
            "trunk: {} ({})",
            TRUNK_VERSION,
            tools.trunk.to_string_lossy()
        ),
        format!(
            "wasm-pack: {} ({})",
            WASM_PACK_VERSION,
            tools.wasm_pack.to_string_lossy()
        ),
    ])
}

pub(crate) fn run(command: &mut Command, label: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("spawning {label}"))?;
    if status.success() {
        Ok(())
    } else {
        bail!("{label} failed with status {status}; inspect the command output and retry")
    }
}

fn version_matches(output: &str, expected: &str) -> bool {
    output.split_whitespace().any(|token| token == expected)
}

fn require_pinned_tool(name: &str, expected: &str) -> Result<PathBuf> {
    let path = resolved_tool(name).with_context(|| {
        format!(
            "{name} {expected} is missing; run just install-tools before building extension artifacts"
        )
    })?;
    let output = Command::new(&path)
        .arg("--version")
        .output()
        .with_context(|| format!("running {name} --version"))?;
    if !output.status.success() {
        bail!("{name} --version failed; reinstall {name} {expected} with just install-tools");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if version_matches(&stdout, expected) || version_matches(&stderr, expected) {
        Ok(path)
    } else {
        let found = format!("{} {}", stdout.trim(), stderr.trim())
            .trim()
            .to_owned();
        bail!("{name} version mismatch: expected {expected}, found {found}; run just install-tools")
    }
}

fn resolved_tool(name: &str) -> Option<PathBuf> {
    let executable_names =
        executable_names(name, std::env::var_os("PATHEXT").as_deref(), cfg!(windows));
    search_path(&executable_names)
        .or_else(|| cargo_home_bin().and_then(|dir| search_dir(&dir, &executable_names)))
}

fn search_path(executable_names: &[OsString]) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|path| search_dir(&path, executable_names))
    })
}

fn cargo_home_bin() -> Option<PathBuf> {
    cargo_home_bin_from(
        std::env::var_os("CARGO_HOME"),
        std::env::var_os("HOME"),
        std::env::var_os("USERPROFILE"),
    )
}

fn cargo_home_bin_from(
    cargo_home: Option<OsString>,
    home: Option<OsString>,
    userprofile: Option<OsString>,
) -> Option<PathBuf> {
    if let Some(home) = cargo_home {
        return Some(PathBuf::from(home).join("bin"));
    }
    if let Some(home) = home {
        return Some(PathBuf::from(home).join(".cargo").join("bin"));
    }
    userprofile.map(|home| PathBuf::from(home).join(".cargo").join("bin"))
}

fn search_dir(dir: &Path, executable_names: &[OsString]) -> Option<PathBuf> {
    executable_names
        .iter()
        .map(|name| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn executable_names(name: &str, pathext: Option<&OsStr>, is_windows: bool) -> Vec<OsString> {
    let mut names = vec![OsString::from(name)];
    if !is_windows || Path::new(name).extension().is_some() {
        return names;
    }

    let raw_exts = pathext
        .and_then(OsStr::to_str)
        .unwrap_or(".COM;.EXE;.BAT;.CMD");
    for ext in raw_exts.split(';').filter(|ext| !ext.is_empty()) {
        let suffix = if ext.starts_with('.') {
            ext.to_owned()
        } else {
            format!(".{ext}")
        };
        names.push(OsString::from(format!("{name}{suffix}")));
    }
    names
}

#[cfg(test)]
mod tests {
    use super::version_matches;

    #[test]
    fn pinned_version_requires_an_exact_token() {
        assert!(version_matches("trunk 0.21.14", "0.21.14"));
        assert!(!version_matches("trunk 0.21.140", "0.21.14"));
    }
}
