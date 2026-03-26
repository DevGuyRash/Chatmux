# Extension Test Harness

Minimal manifest for loading the Chatmux UI as a Chrome extension.

## Usage

1. Build the UI: `cd chatmux-ui && trunk build`
2. Copy `dist/*` into this directory
3. Go to `chrome://extensions`, enable Developer Mode
4. Click "Load unpacked" and select this directory
5. Open the extension popup or side panel to test
