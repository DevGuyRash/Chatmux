//! Deterministic content fingerprints for build inputs and staged artifacts.

use anyhow::{Context, Result, bail};
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::config::BUILD_METADATA_FILE;

const SOURCE_INPUTS: [&str; 21] = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "LICENSE",
    "chatmux-common/Cargo.toml",
    "chatmux-common/src",
    "chatmux-core/Cargo.toml",
    "chatmux-core/src",
    "chatmux-export/Cargo.toml",
    "chatmux-export/src",
    "chatmux-adapter-gpt/Cargo.toml",
    "chatmux-adapter-gpt/src",
    "chatmux-adapter-gemini/Cargo.toml",
    "chatmux-adapter-gemini/src",
    "chatmux-adapter-grok/Cargo.toml",
    "chatmux-adapter-grok/src",
    "chatmux-adapter-claude/Cargo.toml",
    "chatmux-adapter-claude/src",
    "chatmux-ui",
    "extension-src",
    "xtask",
];

/// Fingerprints every source input capable of changing packaged runtime output.
pub(crate) fn source_fingerprint(repo: &Path) -> Result<String> {
    let files = collect_input_files(repo, &SOURCE_INPUTS)?;
    if files.is_empty() {
        bail!(
            "source fingerprint has no inputs; check the repository root and build configuration"
        );
    }
    fingerprint_files(repo, &files)
}

/// Fingerprints every staged runtime file except the self-describing metadata.
pub(crate) fn artifact_fingerprint(dist_dir: &Path) -> Result<String> {
    let mut files = walk_files(dist_dir)?;
    files.retain(|path| {
        path.file_name()
            .is_none_or(|name| name != BUILD_METADATA_FILE)
    });
    if files.is_empty() {
        bail!("artifact fingerprint has no inputs; build and stage the extension first");
    }
    fingerprint_files(dist_dir, &files)
}

fn collect_input_files(root: &Path, inputs: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for relative in inputs {
        let input = root.join(relative);
        if input.is_file() {
            files.push(input);
        } else if input.is_dir() {
            files.extend(walk_files(&input)?);
        }
    }
    files.sort_by(|left, right| {
        normalized_relative(root, left).cmp(&normalized_relative(root, right))
    });
    files.dedup();
    Ok(files)
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !ignored_entry(entry))
    {
        let entry = entry.with_context(|| format!("walking build input {}", root.display()))?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort_by(|left, right| {
        normalized_relative(root, left).cmp(&normalized_relative(root, right))
    });
    Ok(files)
}

fn ignored_entry(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    matches!(
        entry.file_name().to_str(),
        Some(
            ".git"
                | ".local"
                | "target"
                | "node_modules"
                | "dist"
                | "pkg"
                | "extension-dist"
                | "extension-packages"
                | "test-results"
                | "playwright-report"
        )
    )
}

fn fingerprint_files(root: &Path, files: &[PathBuf]) -> Result<String> {
    let mut hash = Fnv1a64::new();
    for path in files {
        let relative = path.strip_prefix(root).with_context(|| {
            format!(
                "fingerprinting {} failed because it is outside {}",
                path.display(),
                root.display()
            )
        })?;
        hash.update(normalized_path(relative).as_bytes());
        hash.update(&[0]);

        let file = fs::File::open(path)
            .with_context(|| format!("opening build input {}", path.display()))?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let count = reader
                .read(&mut buffer)
                .with_context(|| format!("reading build input {}", path.display()))?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        hash.update(&[0xff]);
    }
    Ok(format!("fnv1a64:{:016x}", hash.finish()))
}

fn normalized_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(normalized_path)
        .unwrap_or_else(|_| normalized_path(path))
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

struct Fnv1a64(u64);

impl Fnv1a64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET_BASIS)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::source_fingerprint;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("chatmux-fingerprint-{nonce}"));
        fs::create_dir_all(root.join("chatmux-common/src")).expect("create fixture tree");
        root
    }

    #[test]
    fn source_fingerprint_changes_with_source_content() {
        let root = temp_repo();
        fs::write(root.join("Cargo.toml"), "first").expect("write first source");
        let before = source_fingerprint(&root).expect("fingerprint first source");
        fs::write(root.join("Cargo.toml"), "second").expect("write second source");
        let after = source_fingerprint(&root).expect("fingerprint second source");
        fs::remove_dir_all(&root).expect("remove fixture tree");

        assert_ne!(before, after);
    }
}
