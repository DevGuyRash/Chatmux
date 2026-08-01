//! Deterministic ZIP creation and byte-for-byte archive qualification.

use anyhow::{Context, Result, bail};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub(crate) fn create(source_dir: &Path, archive_path: &Path) -> Result<()> {
    let files = sorted_files(source_dir)?;
    if files.is_empty() {
        bail!("extension archive has no files; stage the extension before packaging");
    }
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating archive directory {}", parent.display()))?;
    }
    let file = fs::File::create(archive_path)
        .with_context(|| format!("creating archive {}", archive_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for path in files {
        let name = normalized_relative(source_dir, &path)?;
        zip.start_file(name, options)?;
        let mut input = fs::File::open(&path)
            .with_context(|| format!("opening staged file {}", path.display()))?;
        std::io::copy(&mut input, &mut zip)
            .with_context(|| format!("archiving staged file {}", path.display()))?;
    }
    zip.finish()?;
    Ok(())
}

pub(crate) fn validate_matches(source_dir: &Path, archive_path: &Path) -> Result<()> {
    let expected = sorted_files(source_dir)?;
    let file = fs::File::open(archive_path)
        .with_context(|| format!("opening archive {}", archive_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("reading archive {}", archive_path.display()))?;
    if archive.len() != expected.len() {
        bail!(
            "archive {} contains {} entries but staged extension contains {}; rebuild the package",
            archive_path.display(),
            archive.len(),
            expected.len()
        );
    }

    for (index, expected_path) in expected.iter().enumerate() {
        let expected_name = normalized_relative(source_dir, expected_path)?;
        let mut entry = archive.by_index(index)?;
        if entry.is_dir() || entry.enclosed_name().is_none() {
            bail!(
                "archive {} contains an unsafe or unexpected entry at index {index}",
                archive_path.display()
            );
        }
        if entry.name() != expected_name {
            bail!(
                "archive entry order/content mismatch: expected {expected_name}, found {}; rebuild the package",
                entry.name()
            );
        }
        let expected_bytes = fs::read(expected_path)
            .with_context(|| format!("reading staged file {}", expected_path.display()))?;
        let mut archived_bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut archived_bytes)?;
        if archived_bytes != expected_bytes {
            bail!(
                "archive entry {expected_name} differs from the staged extension; rebuild the package"
            );
        }
    }
    Ok(())
}

fn sorted_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root) {
        let entry =
            entry.with_context(|| format!("walking staged extension {}", root.display()))?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort_by_key(|path| path.to_string_lossy().replace('\\', "/"));
    Ok(files)
}

fn normalized_relative(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .with_context(|| {
            format!(
                "archive input {} is outside staged root {}",
                path.display(),
                root.display()
            )
        })?
        .to_string_lossy()
        .replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::{create, validate_matches};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn archive_entries_are_sorted_and_exact() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("chatmux-archive-{nonce}"));
        let source = root.join("source");
        fs::create_dir_all(source.join("nested")).expect("create archive fixture");
        fs::write(source.join("z.txt"), "last").expect("write z");
        fs::write(source.join("nested/a.txt"), "first").expect("write a");
        let archive = root.join("artifact.zip");

        create(&source, &archive).expect("create archive");
        validate_matches(&source, &archive).expect("validate archive");
        fs::remove_dir_all(root).expect("remove archive fixture");
    }
}
