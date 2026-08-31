# Kura Rebrand Inventory (from renatobardi/kura)

Generated 2026-08-30. Phase 1 = user-visible. Phase 2 = internal (crate names, DB tables, env vars, docker images, k8s objects).

## 1. Desktop (`desktop/`)

### 1.1 Theme tokens — PHASE 1 (visual) / naming PHASE 2

`--buzz-*` CSS custom properties defined in `desktop/src/shared/styles/globals/theme.css` (30 vars): `--buzz-active-fill`, `--buzz-active-foreground`, `--buzz-channel-fg`, `--buzz-chrome-foreground`, `--buzz-content-dark`, `--buzz-dm-fg`, `--buzz-gradient-bottom`, `--buzz-gradient-dark-bottom`, `--buzz-gradient-dark-top`, `--buzz-gradient-light-bottom`, `--buzz-gradient-light-top`, `--buzz-gradient-top`, `--buzz-hosted-community-action-bg`, `--buzz-hosted-community-action-bg-hover`, `--buzz-hosted-community-divider-border`, `--buzz-hosted-community-identity-bg`, `--buzz-hosted-community-input-bg`, `--buzz-hosted-community-modal-action-fg`, `--buzz-hosted-community-modal-overlay-bg`, `--buzz-hosted-community-surface-fg`, `--buzz-hover-surface`, `--buzz-huddle-drawer-inset`, `--buzz-huddle-drawer-radius`, `--buzz-huddle-drawer-surface`, `--buzz-muted-foreground`, `--buzz-nav-fg`, `--buzz-search-surface`, `--buzz-sidebar-scrollbar-thumb`, `--buzz-translucency-gradient-alpha`, `--buzz-translucency-wash-alpha`.

Consumer file counts (occurrences of `--buzz-`):

| File | Count |
|---|---:|
| `desktop/src/shared/styles/globals/animations.css` | 107 |
| `desktop/src/shared/styles/globals/theme.css` | 100 |
| `desktop/src/shared/styles/globals/components.css` | 89 |
| `desktop/src/features/messages/ui/DiffViewer.css` | 31 |
| `desktop/src/shared/ui/card-texture.css` | 24 |
| `desktop/src/features/profile/ui/ProfileAvatarEditor.utils.ts` | 22 |
| `desktop/src/shared/styles/globals/typography.css` | 20 |
| `desktop/src/shared/styles/globals/composer.css` | 20 |
| `desktop/src/features/profile/ui/ProfileAvatarEditor.tsx` | 20 |
| `desktop/src/features/onboarding/ui/CommunityOnboardingFlow.tsx` | 17 |
| `desktop/src/features/communities/ui/HostedCommunityOnboarding.tsx` | 17 |
| `desktop/src/features/onboarding/ui/SetupStep.tsx` | 13 |
| `desktop/src/shared/styles/globals/markdown.css` | 12 |
| `desktop/src/shared/lib/fontSizePreference.test.mjs` | 12 |
| `desktop/src/shared/layout/chromeLayout.ts` | 9 |
| `desktop/src/shared/styles/globals/avatar-framing.css` | 8 |
| `desktop/src/features/settings/ui/AppearanceSettingsControls.tsx` | 8 |
| `desktop/src/shared/ui/PoofBurstProvider.tsx` | 6 |
| `desktop/src/shared/theme/ThemeProvider.tsx` | 6 |
| `desktop/src/shared/styles/globals/spoilers.css` | 6 |

Related surface/animation components: `desktop/src/app/BuzzThemeSurfaces.tsx`, `desktop/src/app/ThemeGrainientBackground.tsx` — PHASE 1 (rename component + `data-buzz-sidebar` attribute, filenames PHASE 2).

Theme directory `desktop/src/shared/theme/`: `CommunityThemeController.tsx`, `useThemePreviewVars.ts`, `adaptive-theme.ts`, `useSystemColorScheme.ts`, `ThemePreviewFrame.tsx`, `communityThemePreference.ts` (+test), `terminal-palette.ts` (+test), `ThemeProvider.tsx`, `communityThemeSync.ts` (+test), `theme-loader.ts`.

**Community/syntax theme catalog** (`desktop/src/shared/theme/theme-loader.ts`, `SYNTAX_THEMES`): first-party branded entries are `buzz` and `buzz-dark` (Shiki-based; `buzz` = GitHub Light palette + branded sidebar gradient, `buzz-dark` = GitHub Dark equivalent). Constants: `KURA_THEME_NAME = "buzz"`, `KURA_DARK_THEME_NAME = "buzz-dark"`, `KURA_BASE_THEME`, `KURA_DARK_BASE_THEME`. Rest of catalog is third-party Shiki theme names (andromeeda, aurora-x, ayu-dark, catppuccin-*, dark-plus, dracula, dracula-soft, everforest-*, github-*, etc.) — **not** rebrand targets. — PHASE 1 (the "Buzz"/"Buzz Dark" theme display name + gradient identity) / PHASE 2 (internal string ids `buzz`/`buzz-dark`).

### 1.2 Fonts — PHASE 1 (no change needed, but note for asset list)
`desktop/src/main.tsx`: `@fontsource-variable/inter/opsz.css`, `opsz-italic.css`, `@fontsource/jetbrains-mono/400.css`, `700.css`. Other font-family refs in `terminal.css`, `theme.css`, `animations.css`, `markdown.css`. Not brand-specific fonts (Inter, JetBrains Mono) — no rename needed unless brand style guide differs.

### 1.3 Image/icon assets — PHASE 1

`desktop/public/`:
| File | Size |
|---|---:|
| `app-icon@2x.png` | 4.0K |
| `app-icon@3x.png` | 8.0K |
| `buzz.svg` | 4.0K |
| `landing/buzz-wordmark.png` | 328K |
| `onboarding/starter-team/fizz.png` | 1.1M |
| `onboarding/starter-team/honey.png` | 1.0M |
| `onboarding/starter-team/pollen.png` | 1.2M |
| `pow/*` (poof1-5@3x.png, plop.m4a, LICENSE.txt) | 12K-36K each |
| `sounds/*.mp3` + matching `.svg` (bong, boo, dng, doo, doodone, doong, doop, flirl, flutter, oh-no, ping, unison) | 16-28K each |
| `boot.css`, `worklet.js` | 4.0K |
| `harness-logos/*` (amp.png, devin.svg, grok.svg, hermes.png, kimi.png, omp.svg, openclaw.svg, opencode.svg) | third-party logos, not Buzz brand |
| `runtime-icons/claude.png` | third-party |

Note: `buzz.svg` and `landing/buzz-wordmark.png` are the clearest brand-mark assets needing replacement.

`desktop/src-tauri/icons/`: `icon.png`, `icon.icns`, `icon.ico`, `Square30x30Logo.png` … `Square310x310Logo.png`, `StoreLogo.png`, `ios/AppIcon-*.png` (24 files), `android/mipmap-*/ic_launcher*.png` — full Tauri/mobile icon set, all need regeneration from new logo — PHASE 1.

`desktop/src/assets/`: empty/not present.

Also: `desktop/src/shared/ui/buzz-logo/` component directory (`BuzzLogoAnimation.tsx`, `FuzzyLogo.tsx`) — PHASE 1.

### 1.4 Tauri config — PHASE 1 (naming) / PHASE 2 (identifier, binaries)

`desktop/src-tauri/tauri.conf.json`:
- `productName`: `"Buzz"`
- `identifier`: `"xyz.block.buzz.app"`
- `bundle.icon`: `icons/32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`
- `bundle.externalBin`: `binaries/buzz-acp`, `binaries/buzz-agent`, `binaries/buzz-backend-kubernetes`, `binaries/buzz-dev-mcp`, `binaries/git-credential-nostr`, `binaries/buzz`
- `plugins.deep-link.desktop.schemes`: `["buzz"]` → **custom protocol `buzz://`**
- `plugins.updater.endpoints`: `[]` (empty — no live updater URL currently pointing at a Buzz/Block domain)
- CSP (`app.security.csp`): references custom scheme `buzz-media:` and `http://buzz-media.localhost` in `connect-src`, `img-src`, `media-src`

`desktop/src-tauri/tauri.windows.conf.json`: (checked, no productName/identifier override found beyond base)
`desktop/src-tauri/tauri.dev.conf.json`: `identifier: "xyz.block.buzz.app.dev"`, `productName: "Buzz Dev"`

### 1.5 Package names — PHASE 2 (internal) but affects window titles / about dialogs (PHASE 1 adjacent)
- `desktop/src-tauri/Cargo.toml`: `name = "buzz-desktop"` (bin), `name = "buzz_lib"` (lib)
- `desktop/package.json`: `"name": "buzz"`

### 1.6 User-visible "Buzz" strings in `desktop/src/**/*.tsx`

Total JSX-text/string-literal matches (`grep -rn '"[^"]*Buzz[^"]*"\|>[^<]*Buzz' --include=*.tsx desktop/src`): **109**.
Total raw "Buzz"/"buzz" occurrences across all `.ts`/`.tsx` (identifiers, imports, URLs, comments, tests included): **1639** — the 109 above is the user-visible subset; the remainder is PHASE 2 (identifiers) or incidental (comments/tests).

Top 25 `.tsx` files by user-visible-string-match count:

| Count | File |
|---:|---|
| 12 | `desktop/src/features/settings/ui/HostedCommunitiesSettingsCard.tsx` |
| 7 | `desktop/src/features/profile/ui/NostrBindConsentDialog.tsx` |
| 7 | `desktop/src/features/communities/ui/HostedCommunityOnboarding.tsx` |
| 6 | `desktop/src/features/terminal/TerminalSubstrate.tsx` |
| 5 | `desktop/src/features/onboarding/ui/BackupStep.tsx` |
| 5 | `desktop/src/features/communities/ui/HostedCommunityCreateFlow.tsx` |
| 4 | `desktop/src/features/projects/ui/ProjectCards.tsx` |
| 3 | `desktop/src/features/projects/ui/ProjectDetailFeedPanels.tsx` |
| 3 | `desktop/src/features/onboarding/ui/MachineOnboardingFlow.tsx` |
| 2 | `desktop/src/shared/ui/buzz-logo/FuzzyLogo.tsx` |
| 2 | `desktop/src/features/projects/ui/RepositoryCards.tsx` |
| 2 | `desktop/src/features/projects/ui/ProjectRightPanelControls.tsx` |
| 2 | `desktop/src/features/onboarding/ui/RuntimeIcon.tsx` |
| 2 | `desktop/src/features/onboarding/ui/RelaunchRequiredScreen.tsx` |
| 2 | `desktop/src/features/channels/ui/ChannelScreenHeader.tsx` |
| 2 | `desktop/src/features/agents/ui/agentConfigOptions.tsx` |
| 1 | `desktop/src/shared/ui/markdown/entityLinks.tsx` |
| 1 | `desktop/src/shared/ui/markdown/MessageLinkPill.tsx` |
| 1 | `desktop/src/shared/ui/markdown/ChannelDeepLink.tsx` |
| 1 | `desktop/src/shared/ui/compact-link-preview-attachment.tsx` |
| 1 | `desktop/src/shared/ui/buzz-logo/BuzzLogoAnimation.tsx` |
| 1 | `desktop/src/shared/ui/ViewLoadingFallback.tsx` |
| 1 | `desktop/src/shared/ui/BuzzLoadingState.tsx` |
| 1 | `desktop/src/features/settings/ui/VoiceSettingsCard.tsx` |
| 1 | `desktop/src/features/settings/ui/SettingsPanels.tsx` |

PHASE 1.

### 1.7 URLs — PHASE 1 (user-facing links) / mixed

Occurrences by directory (repo-wide, not desktop-only, since counted together):

`buzz.xyz`: `desktop/tests/e2e` (3), `admin-web/tests` (3), `docs` (2), `desktop/src/shared/lib` (2), `mobile/test/shared/push` (1), `mobile/test/features/pairing` (1), `mobile/lib/shared/relay` (1), `mobile/ios/BuzzPushKit/Tests/BuzzPushKitTests` (1), `mobile/ios/BuzzPushKit/Sources/BuzzPushKit` (1), `docs/nips` (1).

`block.xyz`: `docs` (2), `mobile/ios/BuzzPushKit/Tests/BuzzPushKitTests` (1), `README.md` (1).

`github.com/renatobardi/kura`: `desktop/tests/e2e` (8), `docs` (3), `desktop/src/features/projects/lib` (3), `desktop/src/shared/lib` (2), `desktop/src-tauri/src/commands` (2), `web/tests/e2e` (1), `web/src/shared/lib` (1), `mobile` (1), `docs/nips` (1), `desktop/src/testing` (1).

### 1.8 Deep link scheme `buzz://` — PHASE 1 (very high risk)

Used across ~48 files, including core: `desktop/src/shared/deep-link.ts`, `desktop/src/shared/useAppDeepLinks.ts`, `desktop/src/shared/useEntityDeepLinks.ts`, `desktop/src/shared/useMessageDeepLinks.ts`, `desktop/src-tauri/src/deep_link.rs`, `desktop/src-tauri/src/lib.rs`, `desktop/src-tauri/src/deep_link_tests.rs`, plus message-link/channel-link/entity-link libs under `desktop/src/features/messages/lib/` and `desktop/src/features/projects/lib/`, and many `.test.mjs` files.

### 1.9 Custom protocol `buzz-media:` — PHASE 2 boundary but affects CSP (PHASE 1 adjacent risk)
Used in: `desktop/src/shared/lib/useMediaProxyPort.ts`, `desktop/src/shared/lib/mediaUrl.ts`, `desktop/src/testing/e2eBridge.ts`, `desktop/src-tauri/Cargo.toml`, `desktop/src-tauri/tauri.conf.json` (CSP), `desktop/src-tauri/tests/csp.rs`, `desktop/src-tauri/src/lib.rs`, `desktop/src-tauri/src/commands/media_gif.rs`, `media.rs`, `media_snapshot_png.rs`, `desktop/src-tauri/src/media_proxy.rs`.

---

## 2. Mobile (`mobile/`)

- `mobile/pubspec.yaml`: `name: buzz`, `description: Buzz mobile client` — PHASE 1/2.
- Android: `mobile/android/app/build.gradle.kts` → `applicationId = "xyz.block.buzz.mobile"` (PHASE 2); `AndroidManifest.xml` → `android:label="@string/app_name"`, but no `values/strings.xml` defining `app_name` was found in this pass (needs follow-up — likely generated/flavored) — PHASE 1.
- iOS: `mobile/ios/Runner/Info.plist` → `CFBundleDisplayName` = `$(APP_DISPLAY_NAME)` (build-setting driven, defined in `mobile/ios/Flutter/Debug.xcconfig` and `Release.xcconfig`) — PHASE 1. Bundle identifier is templated via `$(BUNDLE_IDENTIFIER)` in `project.pbxproj` (actual value set in xcconfig, not found in this pass) — PHASE 2.
- Launcher icons: `mobile/ios/Runner/Assets.xcassets/AppIcon.appiconset`, `mobile/android/app/src/main/res/mipmap-*/ic_launcher*.png` (+`ic_launcher_round.png`, `ic_launcher_foreground.png`), `mobile/android/app/src/main/res/drawable/ic_launcher_foreground_inset.xml`, `mobile/android/app/src/main/res/values/ic_launcher_background.xml`, `mobile/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml` — PHASE 1.

### 2.1 Theme — `mobile/lib/shared/theme/`
Files: `text_theme.dart`, `theme_catalog.dart`, `adaptive_theme.dart`, `community_theme_preference.dart`, `color_scheme.dart`, `grid.dart`, `accent_colors.dart`, `message_typography.dart`, `utility_surface_theme.dart`, `app_theme.dart`, `app_colors.dart`, `buzz_theme.dart`, `theme_provider.dart`, `community_theme_sync.dart`, `theme_pairs.dart`, `community_theme_provider.dart`, `theme_extensions.dart`, `theme.dart`.

Built-in theme catalog (`theme_catalog.dart`) mirrors desktop's Shiki theme list; first-party branded entries are `'buzz'` and `'buzz-dark'` (see desktop 1.1) — same rename scope.

Accent color names (`accent_colors.dart`, `accentColors` list) — generic color names (Neutral, Blue, Cyan, …), not brand-specific — PHASE 1 no-op, listed for completeness.

Fonts (`pubspec.yaml`): Inter (`InterVariable.ttf`, `InterVariable-Italic.ttf`), Geist Mono (`GeistMono-Variable.ttf`, `GeistMono-Italic-Variable.ttf`) — not brand-named, no change needed.

### 2.2 User-visible "Buzz" strings in `mobile/lib/**/*.dart`
Total: **500**. Top 15 files:

| Count | File |
|---:|---|
| 72 | `mobile/lib/shared/push/push_subscription.dart` |
| 48 | `mobile/lib/shared/push/push_lease_revocation_outbox.dart` |
| 39 | `mobile/lib/shared/push/push_bridge.dart` |
| 30 | `mobile/lib/shared/push/dev_push_lease.dart` |
| 18 | `mobile/lib/shared/theme/buzz_theme.dart` |
| 18 | `mobile/lib/shared/push/push_bootstrap.dart` |
| 14 | `mobile/lib/features/channels/agent_activity/transcript_builder.dart` |
| 10 | `mobile/lib/shared/push/push_relay_capability_provider.dart` |
| 9 | `mobile/lib/shared/deeplink/deep_link.dart` |
| 8 | `mobile/lib/features/settings/settings_page/notifications_section.dart` |
| 8 | `mobile/lib/features/channels/channel_details_page.dart` |
| 7 | `mobile/lib/shared/push/push_snapshot.dart` |
| 7 | `mobile/lib/shared/deeplink/pending_deep_link_provider.dart` |
| 7 | `mobile/lib/shared/community/community_provider.dart` |
| 6 | `mobile/lib/shared/widgets/modal_presentation.dart` |

Note: most of this volume is in `push/*` (likely identifiers like `BuzzPushKit`, class/type names — largely PHASE 2) — recommend manual triage of these 5 files before counting as PHASE 1 UI text.

iOS push module `mobile/ios/BuzzPushKit/` (Sources + Tests) is itself Buzz-named — PHASE 2 (internal module), but any user-visible notification strings inside are PHASE 1.

---

## 3. Web (`web/`) and admin-web

- `web/index.html`: `<title>Buzz</title>` — PHASE 1.
- `admin-web/index.html`: `<title>Buzz admin</title>` — PHASE 1.
- `admin-web/public/favicon.svg` — PHASE 1 (asset to replace). No dedicated favicon file found under `web/` in this pass — needs follow-up (may reuse `public/app-icon@*` or none configured).
- `web/src/shared/styles/globals.css`: 0 occurrences of `--buzz-` (no theme tokens duplicated here) — web app does not share desktop's CSS var namespace.

User-visible "Buzz" string counts:

`web/src` (top files): `shared/lib/buzz-download.ts` (15), `features/invite/ui/InvitePage.tsx` (10), `features/repos/ui/ReposPage.tsx` (2), `features/repos/mock-repos.ts` (2), `features/invite/ui/InviteJoinPolicyNotice.tsx` (2), `shared/lib/nostr-client.ts` (1), `features/repos/ui/RepoDetailPage.tsx` (1), `features/repos/ui/OrgSidebar.tsx` (1), `features/repos/ui/ConnectButton.tsx` (1).

`admin-web/src`: `App.tsx` (5) — only file with matches.

PHASE 1.

---

## 4. Docs & repo root

Root markdown files, "Buzz" occurrence counts:

| Count | File |
|---:|---|
| 43 | `CHANGELOG.md` |
| 16 | `README.md` |
| 12 | `VISION_PROJECTS.md` |
| 11 | `VISION.md` |
| 11 | `ARCHITECTURE.md` |
| 9 | `VISION_SOVEREIGN.md` |
| 8 | `VISION_MESH.md` |
| 8 | `NOSTR.md` |
| 7 | `CONTRIBUTING.md` |
| 5 | `VISION_REMOTE_AGENTS.md` |
| 5 | `VISION_MODERATION.md` |
| 5 | `TESTING.md` |
| 4 | `VISION_AGENT.md` |
| 4 | `AGENTS.md` |
| 3 | `SECURITY.md` |
| 3 | `RELEASING.md` |
| 2 | `VISION_ACTIVITY.md` |
| 0 | `GOVERNANCE.md`, `CODE_OF_CONDUCT.md` |

`docs/` top files: `docs/buzz-shared-compute-dev.md` (16 — filename itself Buzz-branded), `docs/nips/NIP-MP.md` (14), `docs/linux-rendering-troubleshooting.md` (12), `docs/buzz-entity-links.md` (12 — filename Buzz-branded), `docs/remote-agents.md` (11), `docs/nips/NIP-CW.md` (10), `docs/nips/NIP-PL.md` (6), `docs/nips/NIP-WP.md` (5), `docs/gif-search.md` (4), `docs/deployment-identity.md` (4).

Screenshots (`docs/assets/screenshots/`): `create-channel.png`, `media-comments.png`, `channel-agents.png`, `channel-thread.png` — all show the app UI and likely need retaking post-rebrand — PHASE 1.

CHANGELOG.md is historical record — recommend leaving as-is (do not rewrite history) even though it scores highest; flag explicitly as an exception.

---

## 5. Deploy/CI/scripts — mostly PHASE 2, some user-facing

- `deploy/compose/*` (compose.yml, compose.dev.yml, compose.caddy.yml, run.sh, .env.example, README.md) — mention `buzz`; compose project name / `.env.example` var `KURA_IMAGE` — PHASE 2, except `deploy/compose/README.md` if user-facing setup doc → PHASE 1 review.
- `deploy/charts/buzz-push-gateway/` — Helm chart named `buzz-push-gateway` (Chart.yaml, values.yaml, values-production.yaml, templates/*, tests/*) — PHASE 2.
- `deploy/charts/buzz/` — Helm chart named `buzz` (Chart.yaml, `buzz.image` template helper, ci/quickstart-values.yaml, tests/*) — PHASE 2.
- Docker image references: `docker-compose.yml` → `image: ${KURA_IMAGE:-ghcr.io/renatobardi/kura:main}`; `deploy/charts/buzz/templates/deployment.yaml` and `pairing-relay.yaml` → `{{ include "buzz.image" . }}` — PHASE 2 (registry path `ghcr.io/renatobardi/kura` is user-visible only to operators pulling the image — borderline, flag for Phase 1 review since it appears in public docs/README install instructions).
- `.github/workflows/`: `sprig-image.yml`, `release.yml`, `helm-chart.yml`, `codex-security-review.yml`, `promote-oss-desktop-release.yml`, `mesh-lifecycle.yml`, `auto-tag-on-release-pr-merge.yml`, `linux-canary.yml`, `windows-canary.yml`, plus `.github/CODEOWNERS` — reference `buzz`; GitHub release names/tags visible to users on the Releases page — PHASE 1 review for release naming, PHASE 2 for workflow internals.

---

## 6. Crates

All 29 first-party crates under `crates/*` (Cargo.toml `[package] name`) — PHASE 2:

`buzz-acp`, `buzz-admin`, `buzz-agent`, `buzz-audit`, `buzz-auth`, `buzz-backend-kubernetes`, `buzz-cli`, `buzz-conformance`, `buzz-core`, `buzz-datastore-tracing`, `buzz-db`, `buzz-deletion`, `buzz-dev-mcp`, `buzz-media`, `buzz-pair-relay`, `buzz-pairing-cli`, `buzz-persona`, `buzz-pubsub`, `buzz-push-gateway`, `buzz-relay-mesh`, `buzz-relay`, `buzz-sdk`, `buzz-search`, `buzz-test-client`, `buzz-voice`, `buzz-workflow`, `buzz-ws-client`.
(Not Buzz-named: `git-credential-nostr`, `git-sign-nostr`, `sprig`.)

### 6.1 User-facing strings — PHASE 1

`crates/buzz-cli/src/lib.rs`:
```
name = "buzz",
about = "Buzz CLI — interact with a Buzz relay",
long_about = "..."
```
This is printed in `--help` output — user-visible.

`crates/buzz-relay/src/nip11.rs` (NIP-11 relay info document, served to any Nostr client querying the relay):
```
name: "Buzz Relay",
description: "Buzz — private team communication relay",
contact: None,
software: "https://github.com/renatobardi/kura",
```
All four fields are externally visible to any relay-info consumer — PHASE 1, high priority (this is the relay's public identity).

---

## Risk notes

Things that will break or need careful coordination if renamed carelessly:

1. **Custom protocol `buzz-media:`** — hardcoded in Tauri CSP (`tauri.conf.json`), Rust media proxy (`desktop/src-tauri/src/media_proxy.rs`, `commands/media*.rs`), and TS consumers (`useMediaProxyPort.ts`, `mediaUrl.ts`). A rename must update CSP `connect-src`/`img-src`/`media-src` in lockstep with the Rust protocol handler registration and every TS string, or media loading silently breaks (CSP violation).
2. **Deep link scheme `buzz://`** — registered in `tauri.conf.json` (`plugins.deep-link.desktop.schemes`), OS-level association (Windows registry / macOS Info.plist via Tauri bundler, Android intent-filter / iOS URL scheme on mobile via `mobile/lib/shared/deeplink/deep_link.dart`), and referenced in ~48 desktop files plus tests. Renaming the scheme breaks any existing external links, bookmarks, and already-shipped app builds' OS registrations; needs a transition period (register both old and new schemes) rather than a hard cutover.
3. **App identifier / bundle ID** — `xyz.block.buzz.app` (desktop), `xyz.block.buzz.mobile` (Android), iOS `$(BUNDLE_IDENTIFIER)` (value not located in this pass — follow up in xcconfig). Changing bundle/application IDs on already-published apps is effectively a **new app** on app stores (loses reviews, install base, update path) — must be an explicit, deliberate decision, likely Phase 2/3, not incidental to a visual rebrand.
4. **Updater endpoints** — currently empty (`plugins.updater.endpoints: []`) in the OSS conf, so no live risk found here, but confirm any private/production overlay config doesn't point to a `buzz`-branded update server that would need parallel migration.
5. **externalBin names** — `binaries/buzz-acp`, `buzz-agent`, `buzz-backend-kubernetes`, `buzz-dev-mcp`, `buzz` (and `git-credential-nostr`) are referenced by exact filename in `tauri.conf.json` bundle config; renaming the crate binaries (Phase 2) requires updating this list atomically or the desktop build fails to find sidecar binaries.
6. **NIP-11 relay info** (`crates/buzz-relay/src/nip11.rs`) — public protocol-level identity queried by any Nostr client; `software` field is a GitHub URL (`github.com/renatobardi/kura`) that will need to point at the new repo, and any client-side allowlists/pinning keyed on relay `name` could break.
7. **Test files asserting "Buzz" strings**: **109 test files** (`.test.*`, `_test.dart`, `*test*.rs` patterns) reference "Buzz" across the repo — renaming user-visible strings without updating these will cause widespread test failures; budget for a dedicated test-fixture pass.
8. **CHANGELOG.md** (43 occurrences) — historical record; recommend NOT rewriting past entries, only using new branding going forward, to avoid falsifying history.
9. **Community/syntax theme id `buzz`/`buzz-dark`** is duplicated independently in desktop (`theme-loader.ts`) and mobile (`theme_catalog.dart`) — must rename both in sync or cross-platform theme-preference sync (`communityThemeSync.ts`, `community_theme_sync.dart`) will desync between platforms for users who picked the "Buzz" theme.
10. **Docker image path `ghcr.io/renatobardi/kura`** and Helm chart names `buzz` / `buzz-push-gateway` are technically Phase 2, but they appear in the public README/deploy docs that self-hosting users copy-paste — a partial rename (app renamed, docs/images not) will break copy-pasted install instructions; sequence Phase 1 docs updates to not reference Phase-2-only identifiers before those are ready.
