use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::env;
use std::ffi::{OsStr, OsString};
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

    copy_extension_assets(&repo, browser, &dist_dir)?;
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

fn copy_extension_assets(repo: &Path, browser: &str, dist_dir: &Path) -> Result<()> {
    copy_asset_tree(&repo.join("extension-src").join("common"), dist_dir, false)?;
    copy_asset_tree(&repo.join("extension-src").join(browser), dist_dir, true)?;
    Ok(())
}

fn copy_asset_tree(source_root: &Path, dist_dir: &Path, skip_manifest: bool) -> Result<()> {
    if !source_root.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(&source_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            continue;
        }
        if skip_manifest && entry.file_name() == "manifest.json" {
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
        rewrite_staged_ui_index(&ui_target)?;
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
        return Ok(());
    }

    if ui_dist.exists() {
        return Ok(());
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
    let executable_names = executable_names(name, env::var_os("PATHEXT").as_deref(), cfg!(windows));
    search_path(&executable_names)
        .or_else(|| cargo_home_bin().and_then(|dir| search_dir(&dir, &executable_names)))
}

fn search_path(executable_names: &[OsString]) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|path| search_dir(&path, executable_names))
    })
}

fn cargo_home_bin() -> Option<PathBuf> {
    cargo_home_bin_from(
        env::var_os("CARGO_HOME"),
        env::var_os("HOME"),
        env::var_os("USERPROFILE"),
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
        .and_then(|value| value.to_str())
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

fn rewrite_staged_ui_index(ui_target: &Path) -> Result<()> {
    let index_path = ui_target.join("index.html");
    if !index_path.exists() {
        return Ok(());
    }

    let html = fs::read_to_string(&index_path)?;
    let rewritten = rewrite_root_absolute_ui_paths(&html);
    let rewritten = externalize_inline_module_bootstrap(ui_target, &rewritten)?;
    if rewritten != html {
        fs::write(index_path, rewritten)?;
    }

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
    const INLINE_MODULE_OPEN: &str = "<script type=\"module\">";
    const INLINE_MODULE_CLOSE: &str = "</script>";
    const BOOTSTRAP_NAME: &str = "bootstrap.js";
    const BOOTSTRAP_TAG: &str = "<script type=\"module\" src=\"./bootstrap.js\"></script>";
    const BOOTSTRAP_MARKERS: [&str; 2] = ["TrunkApplicationStarted", "module_or_path:"];

    let mut search_from = 0usize;
    let mut saw_inline_module_script = false;
    while let Some(script_start_rel) = html[search_from..].find(INLINE_MODULE_OPEN) {
        saw_inline_module_script = true;
        let script_start = search_from + script_start_rel;
        let script_body_start = script_start + INLINE_MODULE_OPEN.len();
        let Some(script_end_rel) = html[script_body_start..].find(INLINE_MODULE_CLOSE) else {
            bail!("staged UI index contains an unterminated inline module script");
        };
        let script_end = script_body_start + script_end_rel;
        let script_body = html[script_body_start..script_end].trim();
        let script_range = script_start..(script_end + INLINE_MODULE_CLOSE.len());

        if BOOTSTRAP_MARKERS
            .iter()
            .all(|marker| script_body.contains(marker))
        {
            fs::write(ui_target.join(BOOTSTRAP_NAME), format!("{script_body}\n"))?;
            return Ok(format!(
                "{}{}{}",
                &html[..script_range.start],
                BOOTSTRAP_TAG,
                &html[script_range.end..]
            ));
        }

        search_from = script_end + INLINE_MODULE_CLOSE.len();
    }

    if !saw_inline_module_script {
        return Ok(html.to_owned());
    }

    bail!("staged UI index did not contain a recognizable Trunk bootstrap module script")
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

#[cfg(test)]
mod tests {
    use super::{
        cargo_home_bin_from, executable_names, externalize_inline_module_bootstrap,
        rewrite_root_absolute_ui_paths,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn executable_names_add_windows_suffixes() {
        let names = executable_names("trunk", Some(".EXE;.CMD".as_ref()), true);
        let names = names
            .iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["trunk", "trunk.EXE", "trunk.CMD"]);
    }

    #[test]
    fn executable_names_keep_explicit_extensions() {
        let names = executable_names("trunk.exe", Some(".EXE;.CMD".as_ref()), true);
        let names = names
            .iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["trunk.exe"]);
    }

    #[test]
    fn cargo_home_bin_uses_userprofile_when_home_is_absent() {
        let resolved = cargo_home_bin_from(None, None, Some(r"C:\Users\chatmux".into()));
        assert_eq!(
            resolved,
            Some(
                PathBuf::from(r"C:\Users\chatmux")
                    .join(".cargo")
                    .join("bin")
            )
        );
    }

    #[test]
    fn rewrite_root_absolute_ui_paths_makes_assets_relative() {
        let html = r#"
<link rel="stylesheet" href="/tokens.css">
<script type="module">
import init from '/chatmux-ui.js';
const wasm = await init({ module_or_path: "/chatmux-ui_bg.wasm" });
</script>
"#;

        let rewritten = rewrite_root_absolute_ui_paths(html);

        assert!(rewritten.contains(r#"href="./tokens.css""#));
        assert!(rewritten.contains("from './chatmux-ui.js'"));
        assert!(rewritten.contains(r#"module_or_path: "./chatmux-ui_bg.wasm""#));
        assert!(!rewritten.contains("href=\"/"));
        assert!(!rewritten.contains("from '/"));
        assert!(!rewritten.contains(": \"/"));
    }

    #[test]
    fn externalize_inline_module_bootstrap_rewrites_html_and_writes_file() {
        let temp_dir = std::env::temp_dir().join(format!(
            "chatmux-xtask-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("create temp dir");

        let html = r#"
<html>
<head>
<script type="module">
import init from './chatmux-ui.js';
const wasm = await init({ module_or_path: './chatmux-ui_bg.wasm' });
dispatchEvent(new CustomEvent("TrunkApplicationStarted", { detail: { wasm } }));
</script>
</head>
</html>
"#;

        let rewritten =
            externalize_inline_module_bootstrap(&temp_dir, html).expect("externalize script");
        let bootstrap =
            fs::read_to_string(temp_dir.join("bootstrap.js")).expect("read bootstrap file");

        assert!(rewritten.contains(r#"<script type="module" src="./bootstrap.js"></script>"#));
        assert!(!rewritten.contains("<script type=\"module\">"));
        assert!(bootstrap.contains("import init from './chatmux-ui.js';"));
        assert!(bootstrap.contains("module_or_path: './chatmux-ui_bg.wasm'"));

        fs::remove_dir_all(temp_dir).expect("remove temp dir");
    }
}
