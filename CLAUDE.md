# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Kura — Briefing para Claude Code

> Leia este arquivo no início de toda sessão. É a fonte de verdade rápida do projeto.
> Para detalhes completos: `KURA-BLUEPRINT.md` e `KURA-MVP-PLAN.md`.

---

## 1. O que é o Kura

Kura (蔵, "armazém") é um **agregador de conhecimento autônomo** — segundo cérebro que trabalha em background. Ingere conteúdo de múltiplas fontes (RSS, email, arquivos, web), processa com LLM, mantém um vault consultável via RAG, e entrega outputs automáticos (daily dashboard em áudio + visual).

**Diferencial:** Privado, roda na infra do Bardi, sem terceiro vendo os dados. Diferente do NotebookLM.

**Paradigma:** Configure as fontes uma vez. O sistema coleta, processa e organiza; você só consome e revisa.

---

## 1.5. Papel deste repositório

**Este repositório (`renatobardi/kura`) é o repo de especificações — fonte de verdade de produto.** Não contém código de produção.

O projeto Kura é composto por 3 repositórios independentes:

| Repositório | Conteúdo | Deploy | CI/CD |
|---|---|---|---|
| [`renatobardi/kura`](https://github.com/renatobardi/kura) | **Specs** (este repo) — blueprint, plano, decisões de produto | — | 6 hooks de governança |
| [`renatobardi/kura-server`](https://github.com/renatobardi/kura-server) | Backend Elixir/Phoenix + SurrealDB | Oracle ARM | 15 hooks + GitHub Actions |
| [`renatobardi/kura-macos`](https://github.com/renatobardi/kura-macos) | App macOS SwiftUI | .dmg + Sparkle | 11 hooks + GitHub Actions |

**O que vive aqui:**
- `KURA-BLUEPRINT.md` — decisões de produto, stack, design system, Kura-chan
- `KURA-MVP-PLAN.md` — escopo do MVP, fases de implementação, checklist
- `CLAUDE.md` (este arquivo) — briefing do projeto para Claude Code
- `.claude/hooks/` — 6 hooks de governança (specs) e implementações de referência para os outros repos
- `.claude/settings.json` — configuração dos hooks
- `Old/` (gitignored) — specs legadas mantidas localmente para referência

**O que NÃO vive aqui:**
- Nenhum código Elixir (fica em `kura-server`)
- Nenhum código Swift (fica em `kura-macos`)
- CI/CD de produção (GitHub Actions fica nos repos de código)
- Deploys (rsync, Docker, Mix Releases, .dmg)

**Fluxo de mudanças:**
Toda mudança de produto ou arquitetura deve passar por PR neste repo (specs) antes de ser implementada nos repos de código.

---

## 2. Stack

### Backend (Oracle ARM Free Tier)
| Componente | Tecnologia |
|---|---|
| Linguagem / framework | Elixir 1.17+ / Phoenix 1.7+ |
| Jobs & scheduling | Oban |
| Banco de dados | SurrealDB (document + graph + FTS + vector) |
| Ingestion pipeline | Elixir Broadway |
| Scraping estático | Trafilatura (Python subprocess) |
| Scraping JS-heavy | Playwright + Chromium (on-demand) |
| Push notifications | Pigeon (APNs + FCM) |

### Frontend
| Plataforma | Tecnologia | Status |
|---|---|---|
| macOS | SwiftUI — menu bar app (roda em background) | **MVP** |
| iOS / iPadiPad | SwiftUI | Fase 2 |

### Serviços externos
| Serviço | Uso |
|---|---|
| Firebase Auth | Autenticação (Sign in with Apple) |
| FCM | Push notifications → APNs |
| Gemini API | LLM primário + embeddings + TTS + multimodal |
| BYOS | Claude, Kimi — configurável pelo usuário |

---

## 3. Arquitetura de alto nível

```
SwiftUI (macOS / iOS)
        │ HTTPS + WebSocket (Phoenix Channels)
        ▼
Oracle ARM — Elixir + Phoenix + Oban + SurrealDB
        │
        ▼
Firebase Auth · Gemini API · FCM
```

Agents Elixir rodam como processos OTP supervisionados. Cada collector é um GenServer/Broadway pipeline. SurrealDB é o único banco — não há Postgres, Redis, ou Elasticsearch separados.

---

## 4. Escopo do MVP

### Dentro do MVP
- Backend Elixir completo: API REST + Phoenix Channels + Oban jobs + Broadway pipelines
- SurrealDB schema do vault global
- LLM abstraction layer + Gemini API + BYOS (Claude, Kimi)
- macOS menu bar app (SwiftUI): auth, setup wizard, chat, vault/RAG, daily dashboard, inbox, ⌘K, settings
- Collectors: RSS/feeds, arquivos locais (FSEvents), Google OAuth (Gmail + Drive)
- Firebase Auth + Sign in with Apple
- FCM + APNs push notifications
- CI/CD GitHub Actions completo
- Deploy: Mix Releases + .dmg + Sparkle (auto-update)
- Design system: índigo/藍, SF Symbols thin, Kura-chan (mascote)

### Fora do MVP — não implemente sem instrução explícita
- iOS / iPadOS app
- Microsoft OAuth (Outlook, OneDrive)
- iCloud Mail / Drive
- Web scraping agent autônomo
- Android / Linux / Windows

---

## 5. Infra & Deploy

- **Servidor:** Oracle Cloud ARM (Ampere A1, Free Tier) — Ubuntu 22.04
- **Deploy backend:** `mix release` → artifact → rsync/SSH → systemd restart
- **Deploy macOS:** `.dmg` assinado + Sparkle para auto-update
- **CI/CD:** GitHub Actions — testa, builda, deploya em merge para `main`
- **Env vars:** nunca no código. Sempre via `.env` local (não commitado) ou secrets do GitHub Actions.
- **SurrealDB:** roda como serviço systemd no mesmo ARM. Backup diário via cron.

---

## 6. Workflow de desenvolvimento

Todas as 3 repos (`kura`, `kura-server`, `kura-macos`) seguem o mesmo padrão.

### Branches
```
main          — produção, protegida
develop       — integração
feat/<slug>   — nova feature
fix/<slug>    — correção de bug
chore/<slug>  — infra, deps, config
docs/<slug>   — documentação
refactor/<slug>
perf/<slug>
test/<slug>
ci/<slug>
```
O hook `branch-taxonomy` bloqueia nomes fora deste padrão.

### Commits (Conventional Commits)
```
type(scope): descrição em inglês, imperativo, ≤72 chars

Tipos válidos: feat, fix, docs, style, refactor, perf, test, chore, ci, revert
```
O hook `commit-message` bloqueia commits fora deste formato.

**Importante:** sempre use `-m "..."` direto — o hook parse a string literal. HEREDOC (`-m "$(cat <<'EOF'..."`) faz o hook capturar `$(cat <<'EOF'` como subject e rejeitar o commit.

### Pull Requests
- **Título:** mesmo formato de commit — `type(scope): descrição`
- **Body obrigatório:**
  ```
  ## What
  ## Why
  ## How
  ```
- O hook `pr-standards` bloqueia `gh pr create` sem título e body válidos.
- O hook `code-review-gate` bloqueia `gh pr merge` sem review aprovado (Claude como reviewer automático).

---

## 7. Hooks ativos neste repo

Este repo contém apenas hooks de governança de specs — não há CI de código aqui. Os 6 hooks listados abaixo são configurados em `.claude/settings.json`:

| # | Hook | Evento | Dispara em | Ação |
|---|---|---|---|---|
| 1 | `secret-scanner` | PreToolUse | Write \| Edit | Bloqueia API keys e credenciais nas specs |
| 2 | `env-file-protection` | PreToolUse | Write \| Edit \| Bash | Protege `.env` e bloqueia `git add .env` |
| 3 | `branch-taxonomy` | PreToolUse | Bash | Bloqueia `git checkout -b` fora do padrão |
| 4 | `commit-message` | PreToolUse | Bash | Enforce Conventional Commits |
| 5 | `pr-standards` | PreToolUse | Bash | Valida título e body antes de `gh pr create` |
| 6 | `code-review-gate` | PreToolUse | Bash | Code review via Claude antes de `gh pr merge` |

> Se um hook bloquear, corrija o problema — não tente contorná-lo.

---

## 8. Documentos de referência

| Documento | Conteúdo |
|---|---|
| `KURA-BLUEPRINT.md` | Decisões de produto, stack completa, design system, personagem Kura-chan |
| `KURA-MVP-PLAN.md` | Escopo do MVP detalhado, ordem de implementação, checklist |
| `.claude/settings.json` | Configuração dos 6 hooks |
| `.claude/hooks/` | Código dos 6 hooks de governança |

