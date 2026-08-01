//! Stable build inputs and browser-specific package configuration.

use anyhow::{Result, bail};

pub(crate) const TRUNK_VERSION: &str = "0.21.14";
pub(crate) const WASM_PACK_VERSION: &str = "0.15.0";
pub(crate) const BUILD_METADATA_FILE: &str = "build-metadata.json";
pub(crate) const WASM_CRATES: [&str; 5] = [
    "chatmux-core",
    "chatmux-adapter-gpt",
    "chatmux-adapter-gemini",
    "chatmux-adapter-grok",
    "chatmux-adapter-claude",
];
pub(crate) const PROVIDER_ORIGINS: [&str; 6] = [
    "https://chat.openai.com/*",
    "https://chatgpt.com/*",
    "https://gemini.google.com/*",
    "https://grok.com/*",
    // Host grants are origin-level in Chrome and Firefox; the content script stays path-scoped.
    "https://x.com/*",
    "https://claude.ai/*",
];

/// Browser-specific extension target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Browser {
    Chrome,
    Firefox,
}

impl Browser {
    /// Returns the stable directory and archive label for this browser.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
        }
    }

    /// Parses a supported browser label from the command line.
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "chrome" => Ok(Self::Chrome),
            "firefox" => Ok(Self::Firefox),
            _ => bail!("unsupported browser {value}; choose chrome or firefox"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Browser;

    #[test]
    fn browser_names_are_stable() {
        assert_eq!(Browser::Chrome.as_str(), "chrome");
        assert_eq!(Browser::Firefox.as_str(), "firefox");
    }

    #[test]
    fn unsupported_browser_is_rejected() {
        let error = Browser::parse("safari").expect_err("safari is unsupported");
        assert!(error.to_string().contains("choose chrome or firefox"));
    }
}
