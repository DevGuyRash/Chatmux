use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

const CHROME: &str = "chrome";
const FIREFOX: &str = "firefox";

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => print_help(),
        [cmd] if cmd == "help" => print_help(),
        [cmd, browser] if cmd == "dist" => dist(browser),
        [cmd, browser] if cmd == "package" => package(browser),
        [cmd] if cmd == "package-all" => {
            package(CHROME)?;
            package(FIREFOX)
        }
        [cmd] if cmd == "check-tools" => check_tools(),
        [cmd] if cmd == "clean" => clean(),
        _ => bail!("unknown xtask command"),
    }
}

fn print_help() -> Result<()> {
    println!("cargo xtask dist <chrome|firefox>");
    println!("cargo xtask package <chrome|firefox>");
    println!("cargo xtask package-all");
    println!("cargo xtask check-tools");
    println!("cargo xtask clean");
    Ok(())
}

fn dist(browser: &str) -> Result<()> {
    validate_browser(browser)?;
    let repo = repo_root()?;
    let dist_dir = repo.join("extension-dist").join(browser);
    recreate_dir(&dist_dir)?;

    copy_common_assets(&repo, &dist_dir)?;
    write_manifest(&repo, browser, &dist_dir)?;
    stage_ui_shell(&repo, &dist_dir)?;
    stage_optional_wasm_artifacts(&repo, &dist_dir)?;

    Ok(())
}

fn package(browser: &str) -> Result<()> {
    dist(browser)?;
    let repo = repo_root()?;
    let dist_dir = repo.join("extension-dist").join(browser);
    let package_dir = repo.join("extension-packages");
    fs::create_dir_all(&package_dir)?;
    let version = workspace_version(&repo)?;
    let archive = package_dir.join(format!("chatmux-{browser}-{version}.zip"));
    create_zip(&dist_dir, &archive)?;
    Ok(())
}

fn clean() -> Result<()> {
    let repo = repo_root()?;
    for path in ["extension-dist", "extension-packages"] {
        let full = repo.join(path);
        if full.exists() {
            fs::remove_dir_all(full)?;
        }
    }
    Ok(())
}

fn validate_browser(browser: &str) -> Result<()> {
    if matches!(browser, CHROME | FIREFOX) {
        Ok(())
    } else {
        bail!("unsupported browser: {browser}")
    }
}

fn repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask should live inside the workspace root")
}

fn recreate_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn copy_common_assets(repo: &Path, dist_dir: &Path) -> Result<()> {
    let source_root = repo.join("extension-src").join("common");
    for entry in WalkDir::new(&source_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            continue;
        }
        let rel = entry.path().strip_prefix(&source_root)?;
        let target = dist_dir.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &target)
            .with_context(|| format!("copying {}", entry.path().display()))?;
    }
    Ok(())
}

fn write_manifest(repo: &Path, browser: &str, dist_dir: &Path) -> Result<()> {
    let manifest_path = repo
        .join("extension-src")
        .join(browser)
        .join("manifest.json");
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;
    let version = workspace_version(repo)?;
    let rendered = raw.replace("__VERSION__", &version);
    let mut json: Value = serde_json::from_str(&rendered)?;
    json["version_name"] = Value::String(version.clone());
    let body = serde_json::to_string_pretty(&json)?;
    fs::write(dist_dir.join("manifest.json"), body)?;
    Ok(())
}

fn stage_ui_shell(repo: &Path, dist_dir: &Path) -> Result<()> {
    ensure_ui_artifacts(repo)?;
    let ui_dist = repo.join("chatmux-ui").join("dist");
    let ui_target = dist_dir.join("ui");
    if ui_dist.exists() {
        copy_tree(&ui_dist, &ui_target)?;
    } else {
        fs::create_dir_all(&ui_target)?;
        fs::write(ui_target.join("index.html"), placeholder_ui_html())?;
        fs::write(
            ui_target.join("MISSING_UI_BUILD.txt"),
            "trunk was unavailable and no chatmux-ui/dist directory was present.\n",
        )?;
    }
    Ok(())
}

fn stage_optional_wasm_artifacts(repo: &Path, dist_dir: &Path) -> Result<()> {
    ensure_wasm_artifacts(repo)?;
    let target = dist_dir.join("wasm");
    fs::create_dir_all(&target)?;

    let candidates = [
        repo.join("chatmux-core/pkg"),
        repo.join("chatmux-adapter-gpt/pkg"),
        repo.join("chatmux-adapter-gemini/pkg"),
        repo.join("chatmux-adapter-grok/pkg"),
        repo.join("chatmux-adapter-claude/pkg"),
    ];

    let mut copied_any = false;
    for source in candidates {
        if source.exists() {
            copy_tree(&source, &target)?;
            copied_any = true;
        }
    }

    if !copied_any {
        fs::write(
            target.join("MISSING_ARTIFACTS.txt"),
            "Build the Wasm packages before loading the extension package.\n",
        )?;
    }

    Ok(())
}

fn ensure_ui_artifacts(repo: &Path) -> Result<()> {
    let ui_dir = repo.join("chatmux-ui");
    let ui_dist = ui_dir.join("dist");
    if ui_dist.exists() {
        return Ok(());
    }

    if let Some(trunk) = resolved_tool("trunk") {
        run_command(
            Command::new(trunk)
                .arg("build")
                .arg("--release")
                .current_dir(&ui_dir)
                .env_remove("NO_COLOR")
                .env_remove("CLICOLOR")
                .env_remove("CLICOLOR_FORCE"),
            "trunk build",
        )?;
    }

    Ok(())
}

fn ensure_wasm_artifacts(repo: &Path) -> Result<()> {
    let Some(wasm_pack) = resolved_tool("wasm-pack") else {
        return Ok(());
    };

    for crate_name in [
        "chatmux-core",
        "chatmux-adapter-gpt",
        "chatmux-adapter-gemini",
        "chatmux-adapter-grok",
        "chatmux-adapter-claude",
    ] {
        let crate_dir = repo.join(crate_name);
        let pkg_dir = crate_dir.join("pkg");
        if pkg_dir.exists() {
            continue;
        }

        run_command(
            Command::new(&wasm_pack)
                .arg("build")
                .arg("--target")
                .arg("web")
                .arg("--release")
                .current_dir(&crate_dir),
            &format!("wasm-pack build for {crate_name}"),
        )?;
    }

    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
        let rel = entry.path().strip_prefix(source)?;
        let destination = target.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &destination)?;
        }
    }
    Ok(())
}

fn workspace_version(repo: &Path) -> Result<String> {
    let manifest = fs::read_to_string(repo.join("Cargo.toml"))?;
    let value: Value = toml_to_json(&manifest)?;
    value["workspace"]["package"]["version"]
        .as_str()
        .map(str::to_owned)
        .context("workspace.package.version missing")
}

fn toml_to_json(input: &str) -> Result<Value> {
    let value: toml::Value = toml::from_str(input)?;
    Ok(serde_json::to_value(value)?)
}

fn placeholder_ui_html() -> &'static str {
    r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Chatmux</title>
  </head>
  <body>
    <div id="chatmux-root"></div>
    <!-- TODO(frontend): Supply the UI bundle for the extension page/side panel shell. -->
  </body>
</html>
"#
}

fn create_zip(source_dir: &Path, archive_path: &Path) -> Result<()> {
    let file = fs::File::create(archive_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let name = path
            .strip_prefix(source_dir)?
            .to_string_lossy()
            .replace('\\', "/");
        if entry.file_type().is_dir() {
            if !name.is_empty() {
                zip.add_directory(name, options)?;
            }
            continue;
        }

        zip.start_file(name, options)?;
        let mut handle = fs::File::open(path)?;
        let mut bytes = Vec::new();
        handle.read_to_end(&mut bytes)?;
        zip.write_all(&bytes)?;
    }

    zip.finish()?;
    Ok(())
}

fn tool_available(name: &str) -> bool {
    resolved_tool(name).is_some()
}

fn resolved_tool(name: &str) -> Option<PathBuf> {
    search_path(name).or_else(|| {
        cargo_home_bin()
            .map(|dir| dir.join(name))
            .filter(|candidate| candidate.is_file())
    })
}

fn search_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|candidate| candidate.is_file())
    })
}

fn cargo_home_bin() -> Option<PathBuf> {
    if let Some(home) = env::var_os("CARGO_HOME") {
        return Some(PathBuf::from(home).join("bin"));
    }
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo").join("bin"))
}

fn run_command(command: &mut Command, label: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("spawning {label}"))?;
    if status.success() {
        Ok(())
    } else {
        bail!("{label} failed with status {status}")
    }
}

fn check_tools() -> Result<()> {
    for tool in ["trunk", "wasm-pack", "wasm-bindgen", "zip"] {
        println!(
            "{tool}: {}",
            if tool_available(tool) { "yes" } else { "no" }
        );
    }
    Ok(())
}
