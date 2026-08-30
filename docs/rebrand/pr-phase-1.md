## Rebrand — phase 1 (visible surface)

Kura is a new product derived from [block/buzz](https://github.com/block/buzz) @ `eed74bd` (Apache-2.0, see `NOTICE`). This PR replaces everything a user can see of the Buzz brand with the Kura identity (`docs/rebrand/identity.md`): washi/sumi/shu palette, Geist type, hanko seal 蔵 logo.

### Desktop
- Theme tokens light + dark, flat surfaces (no gradients), seigaiha lattice on setup shells
- Geist / Geist Mono / Zen Kaku Gothic New replace Inter / JetBrains Mono
- `KuraMark` / `KuraGlyph` component; every bee mark, the fuzzy "Buzz" wordmark and `buzz-logo/` removed
- User-visible strings, theme labels, help/release URLs → Kura
- `tauri.conf.json`: productName **Kura**, identifier `pro.oute.kura.app`, deep link `kura://` (TS + Rust)
- `Kura Agent` runtime label (`discovery.rs`), Kura accent swatch, matcha presence dot
- Hosted community suffix → `communities.kura.oute.pro` (Block's hosting is not ours)

### Mobile (Flutter) — not build-verified in this PR
- Display name, `pro.oute.kura.mobile`, `kura://` (accepts `buzz://`), theme entries, launcher icons, splash

### Web / admin-web / docs
- Titles, favicon, tokens, Geist, `kura://` links, release URLs; README/VISION/docs prose, Origin section, contacts

### Verification
- desktop: `tsc` clean, unit tests green (8 fixture/expectation updates), `vite build --mode e2e` ok
- 150 screenshot e2e specs: same pass set as upstream (4 flaky `doctor-cta` in both)
- Blind visual A/B critic over 141 screenshot pairs: no Buzz leftover; remaining diffs are pre-existing upstream behaviour
- web e2e 6/6; admin-web 24/28 = upstream baseline

### Deliberately deferred to phase 2
`buzz-*` crate/binary names, `KURA_*` env vars, storage keys, `buzz-media:` protocol, `"Buzz event"` wire prefix (parsers accept both), `push.buzz.xyz`, ghcr image, CI, CHANGELOG, tagline.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_014TU97qPPbxHdEbDnST4P8Q
