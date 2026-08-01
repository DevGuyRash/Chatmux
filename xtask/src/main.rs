//! Command-line entrypoint for deterministic Chatmux extension builds.

use anyhow::{Result, bail};
use std::env;
use std::path::{Path, PathBuf};

mod archive;
mod config;
mod fingerprint;
mod manifest;
mod metadata;
mod pipeline;
mod staging;
mod tools;

use config::Browser;
use pipeline::Pipeline;

fn main() -> Result<()> {
    run(env::args().skip(1))
}

fn run(args: impl IntoIterator<Item = String>) -> Result<()> {
    let args = args.into_iter().collect::<Vec<_>>();
    let pipeline = Pipeline::new(&repo_root()?);
    match args.as_slice() {
        [] => print_help(),
        [command] if command == "help" => print_help(),
        [command] if command == "check-tools" => {
            for line in tools::check_tool_report()? {
                println!("{line}");
            }
            Ok(())
        }
        [command, browser] if command == "dist" => {
            let browser = Browser::parse(browser)?;
            let source = pipeline.build_artifacts()?;
            let path = pipeline.stage(browser, &source)?;
            println!("qualified unpacked extension: {}", path.display());
            Ok(())
        }
        [command, browser] if command == "stage-existing" => {
            let browser = Browser::parse(browser)?;
            let path = pipeline.stage_existing_artifacts(browser)?;
            println!("restaged unpacked extension: {}", path.display());
            Ok(())
        }
        [command] if command == "dist-all" => {
            let source = pipeline.build_artifacts()?;
            pipeline.stage(Browser::Chrome, &source)?;
            pipeline.stage(Browser::Firefox, &source)?;
            pipeline.validate_browser_parity()?;
            println!("qualified unpacked Chrome and Firefox extensions");
            Ok(())
        }
        [command, browser] if command == "package" => {
            let browser = Browser::parse(browser)?;
            let source = pipeline.build_artifacts()?;
            let path = pipeline.package(browser, &source)?;
            println!("qualified extension package: {}", path.display());
            Ok(())
        }
        [command] if command == "package-all" => {
            let source = pipeline.build_artifacts()?;
            pipeline.package(Browser::Chrome, &source)?;
            pipeline.package(Browser::Firefox, &source)?;
            pipeline.validate_browser_parity()?;
            println!("qualified Chrome and Firefox extension packages");
            Ok(())
        }
        [command, browser] if command == "verify-dist" => {
            pipeline.verify_dist(Browser::parse(browser)?)
        }
        [command, browser] if command == "verify-package" => {
            pipeline.verify_existing(Browser::parse(browser)?)
        }
        [command] if command == "verify-all" => {
            pipeline.verify_existing(Browser::Chrome)?;
            pipeline.verify_existing(Browser::Firefox)?;
            pipeline.validate_browser_parity()
        }
        [command] if command == "clean" => pipeline.clean(),
        _ => bail!("unknown xtask command; run cargo run -p xtask -- help"),
    }
}

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("xtask must live inside the Chatmux workspace root"))
}

fn print_help() -> Result<()> {
    println!("cargo run -p xtask -- dist <chrome|firefox>");
    println!("cargo run -p xtask -- stage-existing <chrome|firefox>");
    println!("cargo run -p xtask -- dist-all");
    println!("cargo run -p xtask -- package <chrome|firefox>");
    println!("cargo run -p xtask -- package-all");
    println!("cargo run -p xtask -- verify-dist <chrome|firefox>");
    println!("cargo run -p xtask -- verify-package <chrome|firefox>");
    println!("cargo run -p xtask -- verify-all");
    println!("cargo run -p xtask -- check-tools");
    println!("cargo run -p xtask -- clean");
    Ok(())
}
