# Kura — MVP Scope & Plano de Implementação

> Base: KURA-BLUEPRINT.md. Target: macOS app funcional e distribuível como .dmg.
> TDD Red-Green obrigatório desde a primeira linha de código.

---

## Escopo do MVP (o que entra)

### Infraestrutura
- Oracle ARM: Elixir + Phoenix + Oban + SurrealDB
- Firebase Auth + Sign in with Apple
- FCM push notifications (APNs para macOS)
- CI/CD completo (GitHub Actions)
- Deploy via Mix Releases + .dmg + Sparkle

### Backend
- API REST + Phoenix Channels (real-time para app)
- Supervisor tree para agents autônomos
- SurrealDB schema completo do vault global
- LLM provider abstraction + Gemini API + model router
- BYOS configurável (Gemini + Claude no MVP)
- Ingestion pipeline (Elixir Broadway)
- Oban jobs: collectors, dashboard generator, lint, consolidate

### App macOS (SwiftUI)
- Menu bar app (roda em background)
- Auth flow (Sign in with Apple → Firebase)
- Setup wizard conduzido por Kura-chan
- Chat: persistente + anônimo + título 3 estágios + anexos
- Vault: ask/RAG com citações + lapidação via chat
- Daily dashboard: visual + áudio
- Inbox (centro de comando assíncrono)
- ⌘K command palette
- Settings completo (AI, Connections, Privacy, Notifications, Knowledge, Account, Advanced)
- Cost tracking dashboard

### Collectors (MVP)
- RSS / feeds (sem auth)
- Arquivos locais (FSEvents)
- Google OAuth → Gmail + Google Drive

### Governança básica
- Auto-redação de credenciais e dados sensíveis
- Exclusão por fonte (configuração conversacional)
- Índice de confiabilidade cross-referência

### Design & Personagem
- Design system SwiftUI: índigo/藍, materials nativos, SF Symbols thin
- Kura-chan: loading/thinking + empty states essenciais
- Splash screen noren (primeira abertura do dia)
- Hanko stamp egg (mínimo viável de personalidade)

### O que fica fora do MVP
| Item | Motivo |
|---|---|
| iOS / iPad app | Fase 2 |
| Microsoft OAuth (Outlook, OneDrive) | Fase 2 |
| iCloud Mail / Drive | Fase 2 |
| Web scraping agent | Fase 2 |
| Vault por Space / Project | Fase 2 |
| Spaces com vault | Fase 2 |
| Promote Space → Project | Fase 2 |
| Templates de ecossistema | Fase 2 |
| Weekly digest, entity profiles, deep dive áudio | Fase 2 |
| MCP / ACP | Pós-MVP |
| Egg universe pop culture completo | Fase 2 (só hanko no MVP) |
| Embeddings locais no ARM | Fase 2 (Gemini API no MVP) |

---

## Estrutura de Projeto

### Backend (Elixir)
```
kura_backend/
├── lib/kura/
│   ├── accounts/          # Auth, usuários, Firebase tokens
│   ├── chat/              # Chat, mensagens, título automático
│   ├── vault/             # Pages, FTS, vector, Karpathy wiki
│   ├── collectors/        # RSS, Gmail, Drive, local files
│   │   ├── rss_agent.ex
│   │   ├── gmail_agent.ex
│   │   ├── drive_agent.ex
│   │   └── local_watcher.ex
│   ├── llm/               # Provider abstraction, model router
│   │   ├── provider.ex    # Behaviour (contrato)
│   │   ├── gemini.ex
│   │   ├── claude.ex
│   │   └── router.ex
│   ├── ingestion/         # Broadway pipeline, envelope, redaction
│   ├── outputs/           # Dashboard generator, TTS
│   ├── inbox/             # Async command center
│   ├── governance/        # Auto-redaction, exclusion rules
│   └── notifications/     # FCM / Pigeon
├── lib/kura_web/
│   ├── controllers/
│   ├── channels/          # Phoenix Channels (real-time)
│   └── router.ex
└── test/
    ├── support/
    │   ├── factories/     # ExMachina
    │   ├── mocks/         # Mox behaviours
    │   └── fixtures/
    ├── unit/
    ├── integration/
    └── contract/          # Provider contracts
```

### App macOS (SwiftUI)
```
KuraApp/
├── App/
│   ├── KuraApp.swift      # Menu bar entry point
│   └── AppDelegate.swift
├── Features/
│   ├── Auth/
│   ├── Chat/
│   ├── Vault/
│   ├── Dashboard/
│   ├── Inbox/
│   ├── Search/            # ⌘K
│   └── Settings/
├── Core/
│   ├── API/               # Phoenix Channels + REST client
│   ├── Auth/              # Firebase + Sign in with Apple
│   ├── Models/            # Tipos compartilhados
│   └── Notifications/     # FCM / APNs
└── Design/
    ├── Theme/             # Cores, tipografia, tokens
    ├── Components/        # Botões, cards, toasts, modais
    ├── KuraChan/          # Personagem + animações
    └── Splash/            # Noren animation
```

---

## Fases de Implementação

---

### Fase 0 — Fundação (2–3 semanas)
**Objetivo:** tudo rodando, nada falhando, CI verde desde o primeiro commit.

**Backend:**
- [ ] Repositório Git, GitHub Actions básico (lint + test)
- [ ] Projeto Elixir + Phoenix bootstrapped
- [ ] Conexão com SurrealDB no Oracle ARM
- [ ] Schema inicial: `users`, `sessions`, `vault_pages`, `chats`, `llm_audit`
- [ ] Firebase Auth middleware (valida JWT no header)
- [ ] `GET /api/health` — endpoint de saúde com status dos subsistemas
- [ ] Dialyzer + Credo configurados e no CI
- [ ] Mix Release + Docker para deploy no ARM

**App macOS:**
- [ ] Projeto SwiftUI novo, target macOS, menu bar app
- [ ] Design tokens: cores, tipografia, SF Symbols
- [ ] Sign in with Apple → troca token Firebase → armazena no Keychain
- [ ] Tela pós-login (placeholder)
- [ ] Xcode Cloud ou GitHub Actions com macOS runner

**Testes obrigatórios nesta fase:**
- Auth middleware: token válido passa, inválido retorna 401
- Health endpoint: cada componente reporta status correto
- Schema SurrealDB: migrations aplicadas idempotentemente

**Critério de done:** app macOS abre, usuário faz login com Apple ID, vê tela placeholder, CI verde.

---

### Fase 1 — LLM Layer + Chat (3–4 semanas)
**Objetivo:** conversar com o LLM. O coração do produto funcionando.

**Backend — LLM:**
- [ ] `LLM.Provider` behaviour (contrato Mox-mockável)
- [ ] `LLM.Gemini` — implementação Gemini API (chat, multimodal)
- [ ] `LLM.Claude` — implementação Anthropic API
- [ ] `LLM.Router` — roteia task → modelo correto
- [ ] BYOS config: usuário salva API key por provider, criptografada no SurrealDB
- [ ] `LLM.Audit` — persiste cada chamada tagueada em `llm_audit`

**Backend — Chat:**
- [ ] `Chat` schema: id, title, messages, scope, title_locked, title_msg_count
- [ ] `POST /api/chats` — criar chat
- [ ] `GET /api/chats` — listar
- [ ] `GET|PATCH|DELETE /api/chats/:id`
- [ ] `POST /api/chats/:id/messages/stream` — NDJSON streaming
  - Frames: `start → delta* → done` (+ `error` se falhar)
  - Stub de erro persistido se LLM falhar
- [ ] `POST /api/anon/messages/stream` — stateless, sem persistência
- [ ] Título 3 estágios: heurístico → LLM one-shot → refresher (GenServer background)
- [ ] `POST /api/chats/:id/refresh-title` + `unlock-title`
- [ ] Unicidade global de título (sufixo `(2)`, `(3)`)
- [ ] Upload de anexo: `POST /api/chats/:id/upload` (Gemini processa nativamente)
- [ ] `POST /api/chats/:id/save-to-vault` (chat solto → vault global)

**App macOS — Chat:**
- [ ] Sidebar com lista de chats
- [ ] View de chat: composer, mensagens, streaming
- [ ] Chat anônimo (#/anon)
- [ ] Upload de arquivo (drag-and-drop)
- [ ] Título editável + lock/unlock
- [ ] Settings → AI & Models: configurar providers BYOS

**Testes obrigatórios:**
- `LLM.Provider` behaviour: cada implementação satisfaz o contrato
- Streaming: sequência correta de frames (start → delta → done)
- Streaming com falha: frame error antes do done, stub persistido
- Título: estágio 1 sem LLM, estágio 2 só na 1ª troca, locked não sobrescreve
- Unicidade: colisão gera sufixo correto
- Upload: conteúdo extraído e injetado no prompt
- BYOS: key salva criptografada, router usa provider correto

**Critério de done:** conversa fluindo, título gerado automaticamente, anexo sendo processado.

---

### Fase 2 — Vault Foundation (3–4 semanas)
**Objetivo:** o "segundo cérebro" funcionando. Ingerir, armazenar, consultar.

**Backend — Vault:**
- [ ] Schema completo SurrealDB: `vault_pages`, `vault_pages_history`, `vault_links`, `vault_sources`, `vault_captures`, `ingest_runs`, `vault_issues`
- [ ] FTS indexing no SurrealDB por `title` e `body`
- [ ] Vector indexing (HNSW) no SurrealDB por embedding
- [ ] `LLM.Embeddings` — gera embedding via Gemini Embedding API
- [ ] `Vault.Ingest` — Karpathy wiki processor:
  - Chunking do conteúdo de entrada
  - Embedding de cada chunk
  - FTS + vector search nas páginas existentes (contexto)
  - LLM decide: create / update / no-op
  - Aplica envelope validado
  - Salva histórico de versões
- [ ] `Vault.Ask` — RAG:
  - Embedding da query
  - Busca híbrida (FTS + vector, scores combinados)
  - Top K chunks → contexto para LLM
  - Resposta com citações (`[[página]]`)
  - Cold = 0 resultados → flag no response
- [ ] `POST /api/vault/ingest` — ingestão one-shot
- [ ] `POST /api/vault/ask` — RAG query
- [ ] `GET /api/vault/page` + `GET /api/vault/tree`
- [ ] `GET /api/vault/search` — busca FTS standalone
- [ ] `GET /api/vault/graph` — grafo de links entre páginas
- [ ] `GET /api/vault/backlinks` — backlinks de uma página
- [ ] `GET /api/vault/stats` + `GET /api/vault/health`
- [ ] `POST /api/vault/lint` — job → resultado no Inbox
- [ ] `POST /api/vault/consolidate` — job → resultado no Inbox
- [ ] `POST /api/chats/:id/save-to-vault` — conversa vira capture → ingest

**Backend — Governance básica:**
- [ ] `Governance.Redactor` — detecta e redige API keys, credenciais, dados sensíveis (regex patterns)
- [ ] Redação aplicada em todo conteúdo antes do ingest
- [ ] Exclusão por fonte: regras salvas em SurrealDB, avaliadas no collector

**App macOS — Vault:**
- [ ] View do vault: árvore de páginas, busca, página individual
- [ ] Grafo de conhecimento (visualização)
- [ ] Ask/RAG: input de query, resposta com citações linkáveis
- [ ] Lapidação via chat: "ajusta isso na página X" → LLM aplica

**Testes obrigatórios:**
- Ingest: mesma fonte 2× com hash idêntico → skip (short-circuit)
- Ingest: envelope fora de `wiki/` → EnvelopeError, nada aplicado
- RAG: query com 0 resultados → `cold: true`
- RAG: resposta cita origem das afirmações
- Redactor: API key detectada e substituída por `[REDACTED:api_key]`
- Lint: job termina → issue no Inbox
- Busca híbrida: resultado semântico correto para query sem keyword match

**Critério de done:** ingerir um PDF, fazer pergunta sobre ele, receber resposta com citação.

---

### Fase 3 — Collectors (2–3 semanas)
**Objetivo:** sistema coleta autonomamente. Você não precisa fazer nada.

**Backend — Ingestion Pipeline:**
- [ ] Broadway pipeline: fonte → redação → chunking → embedding → vault ingest
- [ ] Dedup por sha256 (fonte já ingerida com mesmo hash → skip)
- [ ] Estados por item: ingested / duplicate / failed
- [ ] Lock exclusivo por pipeline (evita processamento simultâneo)

**Backend — Agents (GenServers supervisionados):**
- [ ] `Collectors.RSSAgent` — GenServer:
  - Fetch feeds configurados a cada N minutos
  - Parse RSS 2.0 / Atom
  - Envia novos artigos ao pipeline de ingestão
  - Supervisor reinicia automaticamente em crash
- [ ] `Collectors.LocalWatcher` — bridge FSEvents macOS:
  - App macOS detecta mudanças via FSEvents
  - Envia para backend via Phoenix Channel
  - Backend ingere novo arquivo
- [ ] `Collectors.GmailAgent` — GenServer:
  - Google OAuth via Ueberauth
  - Gmail API — busca emails novos (Gmail watch / polling)
  - Candidatos relevantes → Inbox para aprovação
  - Aprovados → pipeline de ingestão
- [ ] `Collectors.DriveAgent` — GenServer:
  - Google Drive API — delta sync de arquivos
  - Novos arquivos → pipeline de ingestão
- [ ] `POST /api/collectors/:type/connect` — OAuth setup
- [ ] `GET /api/collectors` — listar collectors configurados + status

**App macOS:**
- [ ] Settings → Connections: conectar Google (Gmail + Drive com um OAuth)
- [ ] Settings → Connections: adicionar RSS feed
- [ ] Indicador no menu bar: collector ativo / erro
- [ ] Exclusão por fonte: "nunca ingerir emails deste remetente" (conversacional)

**Testes obrigatórios:**
- RSS: feed novo → artigo ingerido no vault; feed repetido → skip por hash
- Gmail agent: crash → supervisor reinicia, retoma coleta
- Local watcher: arquivo adicionado à pasta → detectado e enfileirado
- Pipeline: dois arquivos com mesmo sha256 → 1 ingested, 1 duplicate
- Lock: duas execuções simultâneas → segunda falha com erro claro
- OAuth: token expirado → refresh silencioso, coleta continua

**Critério de done:** RSS coletando automaticamente, Gmail novo email aparecendo no vault, arquivo local detectado e ingerido.

---

### Fase 4 — Outputs & Inbox (2–3 semanas)
**Objetivo:** o sistema te entrega valor proativamente. Você abre o app e já tem algo.

**Backend — Daily Dashboard:**
- [ ] `Outputs.DashboardGenerator` — job Oban agendado (ex: 6h30 todo dia):
  - Agrega: emails relevantes do dia, artigos RSS do dia, updates de Drive
  - Consulta vault para contexto relevante
  - LLM sintetiza dashboard estruturado
  - Persiste resultado em SurrealDB
- [ ] `Outputs.AudioBriefing` — job após dashboard:
  - Texto do dashboard → Gemini TTS
  - Salva arquivo de áudio
  - Notifica via push

**Backend — Inbox:**
- [ ] Schema: `inbox_items` (type, source, payload, status: pending/done/dismissed, created_at)
- [ ] Items gerados por: lint, consolidate, collectors (candidatos), issues, erros
- [ ] `GET /api/inbox` — lista items pendentes
- [ ] `POST /api/inbox/:id/approve` + `/dismiss` + `/snooze`
- [ ] Push notification ao criar item no Inbox

**Backend — Push Notifications:**
- [ ] Pigeon configurado: APNs para macOS + iOS
- [ ] FCM como gateway
- [ ] Eventos que geram push: dashboard pronto, ingest grande concluído, item no Inbox, erro de collector

**App macOS:**
- [ ] Home view: dashboard diário (visual, nativo SwiftUI)
- [ ] Player de áudio do briefing
- [ ] Inbox view: lista de items pendentes, interação conversacional
- [ ] Badge no menu bar quando há items no Inbox
- [ ] Push notifications recebidas + deep link para item

**Testes obrigatórios:**
- Dashboard: job roda no horário, resultado persiste, push enviado
- Inbox: lint job cria item, push disparado, approve aplica mudança, dismiss remove da lista
- TTS: texto → arquivo de áudio gerado (mock Gemini TTS em testes)
- Push: item criado → FCM chamado com payload correto

**Critério de done:** acorda de manhã, push chega, abre app, vê dashboard do dia, ouve briefing, trata 2 items do Inbox.

---

### Fase 5 — App Completo (3–4 semanas)
**Objetivo:** todas as features do MVP funcionando. App utilizável no dia a dia.

**Backend:**
- [ ] ⌘K: `GET /api/search?q=...&scope=...` — busca híbrida global
- [ ] Cost tracking: tags em todos os `llm_audit`, agregações por projeto/tipo
- [ ] `GET /api/costs` — dados para dashboard de custo
- [ ] Setup wizard: estado persistido, retomável
- [ ] Doctor/Verify: `GET /api/health/full` — status detalhado por componente
- [ ] `POST /api/governance/exclusion-rules` — salvar regras de exclusão

**App macOS:**
- [ ] ⌘K command palette: busca + ações, resultados agrupados por tipo
- [ ] Settings → Account: dashboard de custo (por período, por tipo, total)
- [ ] Settings → Privacy: gerenciar regras de exclusão
- [ ] Settings → Advanced: Doctor view com Kura-chan preocupada se algo está vermelho
- [ ] Setup wizard completo conduzido por Kura-chan
- [ ] Onboarding lúdico: LLM pergunta sobre interesses e constrói ontologia
- [ ] Folder sync: iCloud Drive/Kura/Projects + Spaces estrutura criada e mantida

**Testes obrigatórios:**
- ⌘K: busca retorna resultados de vault + chats + files corretamente agrupados
- Cost: cada chamada LLM tagueada com context correto
- Exclusion rules: email de remetente excluído não chega ao pipeline
- Doctor: componente offline → aparece vermelho, push de aviso

**Critério de done:** todos os flows do MVP funcionando end-to-end sem workarounds.

---

### Fase 6 — Design, Polish & Ship (2–3 semanas)
**Objetivo:** app que dá orgulho de usar e distribuir.

**Design system completo:**
- [ ] Todos os tokens de cor implementados como Color assets
- [ ] Componentes SwiftUI: botões (primary/ghost/danger), cards, modais, toasts, tooltips, badges, callouts, tabs, drop zones
- [ ] SF Symbols thin em todos os ícones
- [ ] Dark/light/system theme com sync entre abas
- [ ] `prefers-reduced-motion` respeitado em todas as animações
- [ ] Hiragino Sans como escolha editorial primária

**Kura-chan:**
- [ ] Modelo vetorial final (SwiftUI Canvas, infinitamente escalável)
- [ ] Estados: pensando + trabalhando + feliz + dormindo + apologética + relaxando + esperando
- [ ] Integração no loading de LLM (substitui daruma)
- [ ] Empty states: vault vazio, chats vazios, space vazio, inbox vazia
- [ ] Integração no Doctor (estado preocupado)

**Splash screen:**
- [ ] Noren animation macOS (Metal/Canvas, física de tecido, só na 1ª abertura do dia)
- [ ] Detecção de "primeira abertura do dia" persistida

**Easter egg MVP:**
- [ ] Hanko stamp (save / promoção ao vault)
- [ ] Focus mode kill-switch

**Distribuição:**
- [ ] Build .dmg assinado (Apple Developer certificate)
- [ ] Sparkle integrado (auto-update, verifica servidor de updates)
- [ ] Servidor de updates (pode ser GitHub Releases)
- [ ] Notarização Apple

**Testes finais:**
- [ ] UI testing dos fluxos críticos (XCUITest)
- [ ] Performance: ingest de 100 documentos sem degradação
- [ ] Segurança: paths de vault não podem escapar do escopo
- [ ] Notarização: app passa no Gatekeeper

**Critério de done:** .dmg instalável, auto-update funcionando, app parece produto premium.

---

## Timeline Estimada

| Fase | Semanas | Acumulado |
|---|---|---|
| 0 — Fundação | 2–3 | 3 |
| 1 — LLM + Chat | 3–4 | 7 |
| 2 — Vault Foundation | 3–4 | 11 |
| 3 — Collectors | 2–3 | 14 |
| 4 — Outputs & Inbox | 2–3 | 17 |
| 5 — App Completo | 3–4 | 21 |
| 6 — Polish & Ship | 2–3 | 24 |

**Estimativa total: 18–24 semanas** para MVP completo e distribuível.

---

## Regras de Qualidade (não negociáveis)

1. **TDD Red-Green em toda feature**: teste falhando primeiro, código mínimo para passar, refatorar
2. **Nenhum merge sem CI verde**: testes + Dialyzer + Credo + coverage estável
3. **LLM sempre mockado em testes**: Mox behaviour `LLM.Provider`, respostas determinísticas
4. **Nenhuma secret no código**: API keys só via config/environment, nunca hardcoded
5. **SurrealDB migrations versionadas**: idempotentes, aplicadas no boot
6. **Error sem vazar stack**: respostas de erro genéricas para o cliente
7. **Toda chamada LLM auditada**: `llm_audit` preenchido com contexto antes de retornar resultado

---

## Primeiro PR — O que construir primeiro

O primeiro PR deve ser o menor possível que demonstra a stack funcionando end-to-end:

1. Projeto Elixir + Phoenix no GitHub
2. `GET /api/health` retornando `{status: "ok", surreal: "ok"}`
3. Firebase JWT middleware funcionando
4. GitHub Actions: `mix test` verde

A partir daí, cada PR adiciona uma feature com testes.

---

*Plano criado em 2026-05-29. Próxima revisão: ao final de cada fase.*
