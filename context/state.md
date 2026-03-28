Implement UI runtime bridge and app controller so the extension shell hydrates from the background instead of rendering placeholders.
Replace sidebar/full-tab placeholder branches with mounted workspace, active-workspace, routing, templates, diagnostics, inspection, and settings views.
Add background UI event broadcasting and provider command/event loops so manual send and run flows can drive real content scripts.
Validate packaged Chrome and Firefox behavior with Playwright; pause only if authenticated provider-site testing is required.
WHEN manual browser QA needs a signed-in session THEN use the persistent Chrome stable profile under `~/.config/google-chrome` or the Firefox default-release profile under `~/.mozilla/firefox/n8xi91uc.default-release-1735434875083`, not a Playwright temp profile.
WHEN the target profile is already locked or active THEN avoid a second launch against that profile and reuse the existing browser window/session instead.
Firefox extension e2e launcher now works through `web-ext` with the bundled Playwright Firefox binary, but full Playwright attachment into that launched Firefox extension session remains unresolved.
