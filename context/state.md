Investigate why the guarded live ChatGPT roundtrip does not echo the extension-sent prompt on the provider page even when the ChatGPT surface reports ready.
WHEN manual browser QA needs a signed-in session THEN use the persistent Chrome stable profile under `~/.config/google-chrome` or the Firefox default-release profile under `~/.mozilla/firefox/n8xi91uc.default-release-1735434875083`, not a Playwright temp profile.
WHEN the target profile is already locked or active THEN avoid a second launch against that profile and reuse the existing browser window/session instead.
Firefox extension e2e launcher now works through `web-ext` with the bundled Playwright Firefox binary, but full Playwright attachment into that launched Firefox extension session remains unresolved.
