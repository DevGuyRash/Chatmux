Implement UI runtime bridge and app controller so the extension shell hydrates from the background instead of rendering placeholders.
Replace sidebar/full-tab placeholder branches with mounted workspace, active-workspace, routing, templates, diagnostics, inspection, and settings views.
Add background UI event broadcasting and provider command/event loops so manual send and run flows can drive real content scripts.
Validate packaged Chrome and Firefox behavior with Playwright; pause only if authenticated provider-site testing is required.
