# Chatmux icon sources

These SVGs are the authoritative source for the extension icon. They are **not**
shipped: the staged package contains only the rasterised PNGs in
`extension-src/common/icons/`, which both manifests and the UI favicon reference.

## The mark

Four bars in the provider channel colours — GPT emerald, Gemini azure, Grok
ember, Claude sand — at graduated levels, terminating on the rose operator rail.
It is the product thesis: separate channels, one desk. The same glyph appears in
the app header (`.channel-glyph` in `components.css`), so the icon and the
wordmark are the same idea at two scales.

## Two files, deliberately

| File | Role |
|------|------|
| `chatmux.svg` | 128px master. Used for 32/48/96/128. |
| `chatmux-16.svg` | Hand-tuned for 16px, every coordinate on a whole pixel. |

Scaling the master down to 16px lands the 18-unit bars on fractional pixels and
resamples them to grey mush. The 16px variant redraws the same mark on the pixel
grid so the toolbar icon stays crisp.

## Regenerating the PNGs

Requires `rsvg-convert` (librsvg). Run from the repository root:

```bash
for s in 32 48 96 128; do rsvg-convert -w $s -h $s chatmux-ui/assets/icons/chatmux.svg -o extension-src/common/icons/icon-$s.png; done
rsvg-convert -w 16 -h 16 chatmux-ui/assets/icons/chatmux-16.svg -o extension-src/common/icons/icon-16.png
```

The PNGs are committed rather than built, so the packaging pipeline needs no
image toolchain. `xtask` fails the build if any of the five sizes is missing —
a missing icon does not fail a browser's manifest parse, it silently falls back
to the generic puzzle-piece, so the check has to live in packaging.

Changing a provider channel colour in `tokens.css` means regenerating these.
