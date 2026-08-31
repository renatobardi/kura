# Changelog

## Unreleased

Kura began as a derivative of Kura by Block, Inc. (Apache-2.0) at upstream commit eed74bd; see NOTICE.

- Renamed all crates from `kura-*` to `kura-*` and binaries from `kura*` to `kura*` across the workspace.
- Renamed environment variables from `KURA_*` to `KURA_*` (e.g. `KURA_RELAY_URL`, `KURA_PRIVATE_KEY`, `KURA_SHELL`).
- Repointed deep links, protocol identifiers, and CLI tool descriptions from `kura://` to `kura://`.
- Replaced the "Your people, your agents, your projects — all in one place." tagline with "One storehouse for people and agents." across desktop onboarding, README, and docs.
- Adopted the vermilion hanko seal with the kanji 蔵 ("kura", storehouse) as the product mark.
- Rebranded the repository to `github.com/renatobardi/kura` and container images to `ghcr.io/renatobardi/kura`.
- Swept documentation, agent contributor guides, and CI references from Kura/Block-internal infrastructure to Kura's single self-hosted OSS repo.
- Preserved LICENSE and NOTICE attribution to Block, Inc. and the upstream Kura project.
