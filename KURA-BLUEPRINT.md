# Kura — Blueprint do Produto (v2)

> Documento consolidado de todas as decisões tomadas no redesign do Kura.
> Data: 2026-05-29. Este documento é a fonte da verdade para o novo produto.

---

## 1. Visão do Produto

**O que é:** Um agregador de conhecimento autônomo — "segundo cérebro melhor amigo." Ingere conteúdo de múltiplas fontes, processa com LLM, mantém um vault de conhecimento consultável via RAG, e entrega outputs automáticos diários.

**Diferencial do NotebookLM:** É seu, privado, roda na sua infraestrutura, com seus LLMs, sem nenhum terceiro vendo seus dados.

**Paradigma:** O sistema trabalha por você em background. Você configura as fontes uma vez; ele coleta, processa, organiza e te avisa quando há algo para revisar ou consumir.

---

## 2. Plataformas

| Device | Experiência | Status |
|---|---|---|
| macOS | Menu bar app, roda em background (como Google Drive) | **MVP** |
| iOS | App convencional, foco em consumo e captura rápida | Fase 2 |
| iPadOS | Híbrido — consumo rico + criação moderada | Fase 2 |
| Android | A definir | Futuro |
| Linux | A definir | Futuro |
| Windows | A definir | Futuro |

**Filosofia por device:**
- **Mac:** workspace completo. Gerenciamento de ecossistemas, configuração de collectors, trabalho profundo.
- **iPhone:** consumo e captura rápida. Dashboard do dia, capturar fonte, fazer pergunta, ler output.
- **iPad:** híbrido. Leitura de vault, chat elaborado, adicionar fontes.

---

## 3. Arquitetura

```
┌─────────────────────────────────────────────────────┐
│                  CLIENTES NATIVOS                    │
│   macOS (SwiftUI menu bar)  │  iOS/iPad (SwiftUI)   │
└────────────────┬────────────────────────────────────┘
                 │ HTTPS + WebSocket (Phoenix Channels)
┌────────────────▼────────────────────────────────────┐
│              ORACLE CLOUD ARM (Free Tier)            │
│                                                      │
│  Elixir + Phoenix + Oban                            │
│  ├── API REST                                        │
│  ├── Phoenix Channels (real-time → apps)            │
│  ├── Supervisor Tree (agents autônomos)             │
│  └── Oban (jobs, schedulers, filas)                 │
│                                                      │
│  SurrealDB (document + graph + FTS + vector)        │
│  Trafilatura (Python, web content extraction)       │
│  Playwright + Chromium (JS-heavy pages, on-demand)  │
└─────────────────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────┐
│                  SERVIÇOS EXTERNOS                   │
│  Gemini API (LLM + embeddings + TTS + multimodal)  │
│  Firebase Auth                                      │
│  FCM (push notifications → APNs Apple devices)     │
│  BYOS providers (Claude, Kimi, outros)              │
└─────────────────────────────────────────────────────┘
```

---

## 4. Stack Completa

### Backend
| Componente | Tecnologia | Justificativa |
|---|---|---|
| Framework | Elixir + Phoenix | Agents fault-tolerant via OTP, Channels real-time |
| Jobs/Queue | Oban | Melhor scheduler Elixir — cron, retry, dedup, persistência |
| Banco de dados | SurrealDB | Multi-model: documento + grafo + FTS + vector em um só |
| Web scraping estático | Trafilatura (Python) | Melhor extração de artigos, leve, sem browser |
| Web scraping JS | Playwright + Chromium | On-demand, páginas que requerem rendering |
| Push notifications | Pigeon (Elixir) | APNs + FCM com suporte maduro |

### Frontend
| Componente | Tecnologia |
|---|---|
| macOS | SwiftUI (menu bar app, NSApplicationActivationPolicy.accessory) |
| iOS / iPadOS | SwiftUI (app convencional) |
| Design system | SwiftUI custom components + SF Symbols (thin weight) |

### Infraestrutura
| Componente | Tecnologia |
|---|---|
| Servidor | Oracle Cloud ARM (free tier — 4 OCPUs, 24GB RAM) |
| Auth | Firebase Auth (Sign in with Apple → multi-plataforma futuro) |
| Notificações | FCM → APNs para Apple devices |
| Deploy backend | Mix Releases (binário autocontido) |
| Deploy macOS app | .dmg + Sparkle (auto-update delta, background) |

### LLM & AI
| Componente | Tecnologia |
|---|---|
| LLM primário | Gemini API (free tier start) |
| BYOS providers | Gemini, Claude, Kimi, outros (config estilo Goose) |
| Embeddings | Gemini Embedding API → modelo local no Oracle ARM (fallback) |
| Multimodal | Gemini API nativa (PDF, imagem, áudio, vídeo, YouTube) |
| TTS | Gemini TTS (briefing de áudio diário) |
| Web scraping | Trafilatura (Python) + Playwright/Chromium |

### Fallback cascade de LLM
1. Gemini API
2. Próximo provider BYOS configurado (Claude, Kimi, etc.)
3. Enfileira para retry
4. **Embeddings**: modelo local no Oracle ARM (sempre disponível, independente de API)
5. **PDF/imagem**: Claude API → Poppler/Tesseract local
6. **Áudio**: OpenAI Whisper API → Whisper small local
7. **TTS**: Google Cloud TTS → ElevenLabs

### Model Router
| Task | Modelo padrão |
|---|---|
| Chat, título | Gemini Flash (rápido, barato) |
| Ask/RAG, resumos | Gemini Pro (equilibrado) |
| Vault ingest, consolidate, lint | Gemini Ultra / Claude Opus (alta capacidade) |
| Embeddings | Gemini Embedding API |

Override: `KURA_MODEL_<TASK>` → provider configurado → default da task → `KURA_MODEL_DEFAULT`

---

## 5. Collectors & Fontes de Dados

### MVP
| Fonte | Auth | Tipo |
|---|---|---|
| RSS / feeds | Nenhuma | Background agent, contínuo |
| Arquivos locais | Nenhuma (filesystem) | FSEvents watch |
| Gmail | Google OAuth 2.0 | Background agent |
| Google Drive | Google OAuth 2.0 | Background agent |

### Fase 2
| Fonte | Auth |
|---|---|
| Outlook | Microsoft OAuth |
| OneDrive | Microsoft OAuth |
| iCloud Mail | Nativa macOS/iOS (sem OAuth) |
| iCloud Drive | Nativa macOS/iOS (filesystem) |
| Web scraping agent | Nenhuma (Trafilatura + Playwright) |

### Pós-MVP
| Fonte | Mecanismo |
|---|---|
| Linear, Jira, Slack | MCP connectors |
| Outros | MCP (extensível) |

### OAuth Flow (native apps)
1. `ASWebAuthenticationSession` no app (experiência nativa Apple)
2. Authorization code → exchange por access + refresh token
3. Refresh token: Keychain no device + encrypted para backend Elixir
4. Backend usa refresh token para coleta contínua independente do device
5. Elixir: `Ueberauth` library com estratégias por provider

---

## 6. Vault & Conhecimento

### Fundação: Karpathy Wiki Method
Ingestão incremental mantida por LLM. A cada novo conteúdo:
- LLM lê páginas relevantes existentes + novo conteúdo
- Decide: criar nova página / atualizar existente / no-op
- Vault é sempre integrado, não apenas acumulado

### Estrutura do Vault
- **Global**: vault único compartilhado por todo o sistema
- **Por Project**: vault isolado do projeto (default)
- **Por Space**: vault opcional, sob demanda

### Busca
- **Híbrida**: FTS (keyword precision) + vector (semântica) — ambos no SurrealDB
- **RAG**: embedding da query → busca híbrida → top K chunks → LLM com contexto → resposta com citações

### Features do Vault
| Feature | Tipo |
|---|---|
| Ingest one-shot | On-demand + background |
| Ask / RAG | On-demand, com citações |
| Lapidação via chat | Conversacional — você pede ajustes, LLM aplica |
| Lint | Job background → resultado no Inbox |
| Consolidate | Job background → resultado no Inbox |
| Export | Markdown / HTML / PDF |
| Graph | Visualização nativa (SurrealDB graph) |
| Backlinks | Interno, navegável |
| Health / Stats | Por ecossistema, no dashboard |
| Issues | No Inbox, filtráveis por severidade |
| Inbox de aprovação | Candidatos a ingest (RSS novo, email candidato) |
| Auto-ingest watcher | Opt-in, via FSEvents / coleta contínua |

### Decay Temporal
- Conteúdo antigo não consultado perde peso gradualmente no RAG
- Com o tempo: 10 documentos velhos → 1 resumo consolidado, originais vão para arquivo histórico
- Conteúdo que continua sendo referenciado não decai
- Histórico sempre preservado — nunca deletado

### Confidence Index
- Cada afirmação no vault tem índice de confiabilidade
- Cross-reference 3+ fontes: score cresce
- Fontes conflitantes: flagged para revisão
- Metadado por página: `confidence: 0.0–1.0, sources: [A, B, C]`

---

## 7. Governança & Curadoria

### Onboarding Lúdico
- LLM (via Kura-chan) conversa com o usuário para entender interesses, como pensa sobre entidades, como organiza conhecimento
- Constrói uma ontologia pessoal que guia como o vault é estruturado
- Não é formulário — é conversa natural, progressiva

### Edição do Vault
- Sem edição direta de páginas
- Toda alteração via chat: "lapida isso", "consolida essas notas", "remove o que está errado sobre X"
- LLM executa e registra no histórico

### Exclusão de Dados Sensíveis (3 camadas)
1. **Auto-redação** (sempre ativa): padrões de API keys, credenciais, dados financeiros, CPF — detectados e substituídos por `[REDACTED:tipo]` antes de qualquer ingestão
2. **Regras por fonte**: "nunca ingerir emails deste remetente", "ignorar esta pasta" — configuradas conversacionalmente
3. **Confirmação**: conteúdo detectado como potencialmente sensível pausa e pergunta antes de processar

---

## 8. Ecossistemas — Spaces & Projects

### Conceito
Cada Space e Project é um **ecossistema completo e isolado**:
- Vault próprio
- Chats com histórico
- Collectors configurados individualmente
- System prompt / instructions próprios
- Contexto LLM isolado

### Hierarquia
| Tipo | Descrição | Vault |
|---|---|---|
| **Space** | Exploração sem compromisso de entrega. Pode promover a Project. | Opcional, on-demand |
| **Project** | Estruturado, com objetivo definido. | Sim, por padrão |

### Isolamento (padrão) vs Conexão (opt-in)
- **Isolado** (default): vault do ecossistema não fala com o global
- **Consulta global**: pode perguntar ao vault global, não contribui
- **Contribui ao global**: páginas promovidas ao vault global manualmente
- **Bidirecional**: consulta e contribui

### Atributos
- **Status**: `active → archived → deleted` (soft delete, restaurável)
- **Starred** ⭐: fixar no topo da navegação
- **Tags**: globais, conectadas a entidades do vault (bidirecionais), sugestão por LLM
- **Templates**: criar Space/Project a partir de template pré-configurado

### Promote
- Space → Project: ecossistema completo migra (vault, chats, arquivos)
- Project / Space → Vault global: conteúdo selecionado ou bulk

### Collectors por Ecossistema
- Cada ecossistema pode ter collectors próprios: "este Project vigia esta pasta no Drive"

---

## 9. Organização de Pastas

### Estrutura
```
iCloud Drive/
└── Kura/
    ├── Projects/
    │   ├── Project Alpha/
    │   └── Project Beta/
    └── Spaces/
        ├── Research X/
        └── Ideas Y/
```

### Two-Way Sync
**Kura → Pasta:**
- Criar ecossistema → pasta criada
- Renomear → pasta renomeada
- Arquivar/deletar → pasta move para `Kura/.Archived/`

**Pasta → Kura:**
- Arquivo adicionado à pasta → FSEvents detecta → Kura associa automaticamente
- Nova pasta em `Projects/` ou `Spaces/` → Kura oferece criar ecossistema
- Arquivo removido → Kura reflete

**Por que iCloud Drive:** sync automático para iPhone e iPad, zero infraestrutura extra.

---

## 10. Outputs

| Output | Frequência | Formato |
|---|---|---|
| **Daily dashboard** | Diário | Visual (nativo) + Áudio (TTS) |
| Weekly digest | Semanal | Visual + Áudio |
| Project brief | On-demand / por evento | Documento |
| Entity profiles | Auto-mantidas | Páginas no vault |
| Serendipitous connections | Quando detectadas | Notificação + card |
| Action items extraídos | Contínuo | Inbox |
| Reading list curada | Baseada em interesses | Feed no app |
| Deep dive áudio | On-demand | Áudio (dois personagens) |
| Export | On-demand | Markdown / HTML / PDF |

---

## 11. Inbox — Centro de Comando Assíncrono

Ponto central para tudo que precisa da atenção do usuário:
- Resultados de jobs (lint, consolidate, discuss)
- Candidatos a ingest (RSS, email detectado como relevante)
- Issues do vault por severidade (error / warn / info)
- Erros de collectors
- Conexões serendípicas encontradas

**Fluxo:** Job termina → push notification → usuário abre Inbox → abre item → conversa com Kura sobre o resultado → aprova / rejeita / ajusta.

---

## 12. Chat

| Feature | Descrição |
|---|---|
| Chat persistente | Histórico salvo, streaming NDJSON, anexos, modelo por chat |
| Chat anônimo | Stateless, sem persistência, histórico no cliente |
| Título automático 3 estágios | 1. Heurístico imediato (sem LLM) → 2. LLM one-shot na 1ª troca → 3. Refresher a cada 10 msgs |
| Rename manual | Trava título (`locked`); "destravar" reativa refresher |
| Unicidade de título | Colisão recebe sufixo `(2)`, `(3)` — global entre todos os escopos |
| Upload de anexo | Drag-and-drop no chat, Gemini API processa nativamente (PDF, imagem, etc.) |
| Salvar no vault | Chat solto → vault global; chat em ecossistema → automático via Karpathy |
| Promover ao global | Ação explícita para promover chat de ecossistema ao vault global |
| Lapidação via chat | Ajustar vault conversacionalmente — LLM aplica as mudanças |

---

## 13. Busca Global ⌘K

- **Command palette**: busca + execução de ações
- **Cobertura**: vault global + vaults de ecossistemas + chats + arquivos + entidades + outputs
- **Busca híbrida**: keyword + semântica
- **Resultados agrupados**: Vaults / Chats / Files / Entities / Actions
- **Filtro por ecossistema**: inline na palette
- **iPhone/iPad**: barra de busca nativa no topo, mesma lógica

---

## 14. Settings

| Seção | Conteúdo |
|---|---|
| **Appearance** | Dark/light/system, idioma (EN/pt-BR), focus mode, easter eggs |
| **AI & Models** | Providers BYOS, chaves de API, roteamento por task |
| **Connections** | OAuth conectados (Gmail, Drive, Outlook, OneDrive), RSS feeds |
| **Privacy & Security** | Regras de exclusão, padrões de redação, confirmação antes de ingestão |
| **Notifications** | Preferências de push por tipo e device |
| **Knowledge** | Configurações do vault (decay, confidence, auto-ingest, watcher) |
| **Account** | Firebase Auth, dashboard de custo acumulado |
| **Advanced** | Diagnóstico do sistema (Doctor/Verify), logs, debug |

---

## 15. Cost Tracking

Cada chamada LLM tagueada:
```
task: "vault_ingest"
project_id: "project-alpha"
model: "gemini-flash"
tokens_in: 1240
tokens_out: 380
cost_usd: 0.00042
```

Dashboard no app: custo por projeto, por space, por tipo de output, por período, total acumulado.

---

## 16. Push Notifications (FCM + APNs via Pigeon)

| Evento | Notificação |
|---|---|
| Briefing diário pronto | "Seu briefing de hoje está pronto" |
| Ingestão grande concluída | "X páginas atualizadas no vault" |
| Conexão serendípica encontrada | "Algo novo se conecta a algo antigo" |
| Erro em collector | "Falha ao buscar [fonte]" |
| Job no Inbox aguardando | "Kura-chan tem algo para mostrar" |

---

## 17. Observabilidade

| Ferramenta | Papel |
|---|---|
| Phoenix LiveDashboard | Built-in, real-time: requests, processos, memória, jobs Oban, agents |
| Sentry | Error tracking (cloud, free tier) |
| Structured logging | Arquivos no Oracle ARM com logrotate |
| Firebase Crashlytics | Crash reporting nos apps SwiftUI |

---

## 18. SDLC & DevOps

**Controle de versão:** Git + GitHub, trunk-based development

**CI/CD (GitHub Actions):**
- Backend Elixir: `mix test` + Dialyzer + Credo
- SwiftUI: Xcode Cloud ou GitHub Actions com macOS runner
- Nenhum merge sem: testes passando + Dialyzer verde + Credo limpo + coverage não regredindo

**TDD Red-Green — OBRIGATÓRIO:**
- Teste falhando primeiro (Red)
- Código mínimo para passar (Green)
- Refatorar
- LLM calls sempre mockadas em testes (Mox behaviors)

**Test stack Elixir:** ExUnit + Mox + Bypass + ExMachina + StreamData + Oban Testing

**Test stack SwiftUI:** Swift Testing + XCTest + ViewInspector + XCUITest

**Deploy backend:** Mix Releases (binário autocontido) no Oracle ARM

**Distribuição macOS:** `.dmg` + Sparkle (delta updates, background download, usuário aprova)

---

## 19. Segurança

- Firebase Auth (Sign in with Apple na fase Apple)
- Auto-redação de credenciais antes de qualquer ingestão
- Tokens em Keychain no device
- Refresh tokens criptografados em trânsito para o backend
- Backend acessível apenas via HTTPS autenticado (não é localhost aberto)
- Regras de exclusão por fonte e por conteúdo

---

## 20. State Machines

**Space / Project**
```
active → archived → deleted (soft, restaurável)
```

**Chat title**
```
auto → locked (rename manual) → auto (unlock-title)
```

**Capture (vault)**
```
pending → processing → done
                    → rejected
```

**Ingest run**
```
started → completed
        → failed
```

**Queue file**
```
ingested
duplicate
failed
```

---

## 21. Doctor / Verify

View nativa em **Settings → Advanced**. Verifica:
- Conectividade Oracle ARM
- SurrealDB respondendo
- Firebase Auth ok
- Providers LLM configurados e respondendo
- Collectors OAuth válidos e tokens frescos

Indicadores: verde / amarelo / vermelho por componente. Kura-chan com expressão preocupada se algo está vermelho.

---

## 22. Design System — Opção B (Evoluída)

### Filosofia
"Mesma alma, artesanato elevado." Identidade japonesa intrínseca, não decorativa. Eficiência do ChatGPT preservada na estrutura; artesanato japonês nos detalhes e nas margens da experiência.

### 6 Pilares

**1. Índigo/藍 como identidade própria**
- Accent: `#3730A3` (índigo profundo) / hover: `#4338CA`
- Cor com história cultural japonesa real (藍染め — tingimento índigo)
- Grays de fundo com hint frio, não neutros — conversam com o índigo

**2. Ma/間 — espaço negativo intencional**
- Espaçamento generoso e deliberado
- Empty states como momentos de beleza, não ausência
- Onboarding como cerimônia, não formulário

**3. Hanko/判子 como sistema visual**
- 蔵 como seal em vermelho profundo `#9B1C1C`
- Aparece em: saves, promoções, grandes conquistas no vault
- Único momento quente numa paleta fria

**4. Sazonalidade viva**
- Primavera: hint `#F9A8D4` (rosa sakura) em bordas e empty states
- Verão: leveza, brancos mais presentes
- Outono: `#D97706` âmbar em detalhes
- Inverno: índigo mais fundo e denso
- App icon muda sutilmente por estação

**5. Animações como ritual**
- Ingest concluído → hanko carimbado + haptic
- Space → Project → "Henshin" (transformação)
- Vault update → página "folheada"
- Easter eggs com física real no SwiftUI

**6. Tipografia como arte**
- Hiragino Sans como escolha editorial primária (não fallback) no Apple
- Kanjis em tamanhos grandes em momentos-chave
- Escala tipográfica preservada (fs-2xs → fs-display) com nova `fs-hero`

### Tokens de Cor (Dark Theme)
| Token | Valor | Papel |
|---|---|---|
| `--bg` | `#212121` | Fundo principal |
| `--sidebar` | `#171717` + material | Barra lateral |
| `--surface` | SwiftUI `.regularMaterial` | Cartões/superfícies |
| `--text` | `#ECECEC` | Texto primário |
| `--accent` | `#3730A3` | Índigo — accent principal |
| `--accent-hover` | `#4338CA` | Hover do accent |
| `--hanko` | `#9B1C1C` | Vermelho do seal |
| `--red` | `#EF4444` | Erro |
| `--orange` | `#F59E0B` | Aviso |
| `--green` | `#10B981` | Sucesso |

### Layout
- `NavigationSplitView` (sidebar + conteúdo)
- Sidebar: 260px expandida / colapsável com material translúcido
- Conteúdo: máx 820px, padding generoso
- SF Symbols peso thin (substituem sprite SVG)

---

## 23. Splash Screens

### iPhone / iPad
Animação da bandeira japonesa — 3 segundos, Metal renderer, 120fps:
1. Fundo branco puro
2. Círculo vermelho (`#BC002D`) escala do centro
3. 16 raios emergem em sequência angular — como pétalas abrindo
4. Raios chegam ao limite, pausam, recuam em ondas
5. Fica só o círculo — 日の丸 puro
6. 蔵 aparece dentro do círculo em branco
7. Círculo expande → flash vermelho → app abre
8. Haptic suave na expansão final

### macOS
Cortina noren (暖簾) — 1.2 segundos, SwiftUI Canvas:
- Dois painéis em índigo `#3730A3` com 蔵 em branco
- Painéis se afastam ao centro com física de tecido (SpringAnimation)
- Textura de linho japonês renderizada em Canvas
- App aparece atrás
- **Apenas na primeira abertura do dia** — não repetitivo

---

## 24. Easter Eggs — Camada Pop Culture

Todos opt-in. Focus mode = kill-switch total. Toggle individual em Settings → Appearance → Easter Eggs.

### Kura-chan (Personagem Principal)
Tanuki (狸) kawaii — guardião do armazém. 2.5 cabeças de altura. Kimono curto índigo com mon 蔵 em dourado. Olhos grandes expressivos. Sempre carrega objeto contextual.

**Estados:**
| Estado | Visual | Contexto |
|---|---|---|
| Pensando/Loading | Queixo na mão, bolha com verbos JP typewriter | LLM processing |
| Trabalhando | Correndo com pilha de arquivos | Ingestão em andamento |
| Feliz | Olhos meia-lua, sparkles índigo | Sucesso |
| Dormindo | Enrolada, zzz, lua atrás | Night mode 22:00–05:59 |
| Apologética | Sweat drop, mãos juntas | Erro |
| Relaxando | Rede entre árvores sakura | Inbox vazia |
| Esperando | Olhando horizonte, folhas caindo | Vault vazio |
| Operando gachapon | Girando manivela, olhos brilhando | Egg gachapon |
| Segurando color timer | Olhar preocupado, timer piscando | Cha break / Ultraman |

**Empty states como arte:**
- Vault vazio: dentro de pequeno 蔵 de madeira, prateleiras vazias, luz pela janela
- Chats vazios: num tatami, xícara de chá, esperando
- Space vazio: diante de pergaminho em branco, pincel na mão
- Inbox vazia: numa rede entre árvores de sakura. *"Nada pendente. Kura-chan descansa."*

### Egg Universe

| Egg | Trigger | Descrição |
|---|---|---|
| **Kura-chan loading** | Todo LLM call | Substitui daruma. Bolha com verbos JP typewriter |
| **Kura-chan empty states** | Views vazias | Ilustração única por contexto |
| **Gachapon** | X dias consecutivos / data especial | Kura-chan opera máquina, física real, cápsula com reward aleatório |
| **Color Timer** | 45min ativo / server sob carga | Ultraman color timer, Kura-chan preocupada, haptic sincronizado |
| **Akira — moto** | Konami (↑↑↓↓←→←→ba) / 7 cliques em "kura" | Kaneda's bike com speed lines, high velocity |
| **Akira — explosão psíquica** | Ação de grande impacto no vault | Círculos concêntricos do ponto de toque, 0.4s |
| **Mangá speed lines** | Transição entre seções principais | Linhas convergentes, 0.2s |
| **Painel mangá** | Ingestão grande concluída | 完了 em painel mangá com speed lines |
| **Modo mangá** | Hold 蔵 por 3 segundos | Interface P&B com halftone por 30s |
| **Henshin** | Space → Project promote | Transformação com flash índigo, haptic crescente, nota ascendente. 1.2s |
| **Eye-catch** | Mudança de seção principal | Hanko 蔵 flash 0.25s, efeito câmera analógica |
| **Hanko stamp** | Save / promoção ao vault | Seal 蔵 carimbado + haptic |

---

## 25. Copy & i18n

**Idiomas:**
- EN + pt-BR: app completo, paridade obrigatória (~200 chaves)
- Japonês: momentos culturais (Kura-chan, empty states, eggs) — fragmentos intencionais, não tradução completa

**Duas vozes:**
- **App**: direto, limpo, eficiente, primeira pessoa ("I'll append (2) on save")
- **Kura-chan**: expressiva, levemente dramática no estilo mangá, partículas japonesas ocasionais ("Yosh! 頑張ります", "Ara? Parece vazio por aqui...")

**Onboarding**: lúdico, conversacional, conduzido por Kura-chan. LLM constrói ontologia pessoal do usuário progressivamente.

---

## 26. Roadmap de Features por Fase

### MVP (macOS)
- Backend Elixir + Phoenix + Oban no Oracle ARM
- SurrealDB — schema vault global
- Firebase Auth + Sign in with Apple
- Gemini API (chat, ingestão, embeddings, TTS)
- BYOS básico (Gemini + Claude)
- SwiftUI macOS — menu bar app
- Setup wizard conduzido por Kura-chan
- Chat persistente + anônimo + título 3 estágios + anexos
- Collectors: RSS + arquivos locais + Google OAuth (Gmail + Drive)
- Karpathy wiki — ingestão incremental
- Busca híbrida (FTS + vector no SurrealDB)
- Vault global — ask/RAG com citações
- Daily dashboard visual + áudio
- Inbox — centro de comando assíncrono
- Push notifications básicas (FCM)
- Cost tracking (llm_audit tagueado)
- ⌘K command palette
- Settings completo
- Governança básica (auto-redação, exclusão por fonte)
- Splash screen noren (macOS) e bandeira (iOS)
- Kura-chan loading + empty states
- TDD Red-Green desde o dia 1
- CI/CD completo
- .dmg + Sparkle

### Fase 2 (iOS + iPad + mais collectors)
- Apps iOS + iPadOS SwiftUI
- Microsoft OAuth (Outlook + OneDrive)
- iCloud Mail + iCloud Drive
- Web scraping agent (Trafilatura + Playwright)
- Todos os outputs (weekly digest, entity profiles, connections, deep dive áudio)
- Vault por Space e Project
- Spaces com vault opt-in
- Promote Space → Project com ecossistema completo
- Templates de ecossistema
- Cross-ecosystem query
- Egg universe pop culture completo

### Pós-MVP
- MCP client (Linear, Jira, Slack, etc.)
- MCP server (expõe vault para agentes externos)
- ACP
- Embeddings locais no Oracle ARM
- Mais providers BYOS
- Android / Linux / Windows

---

*Documento gerado em 2026-05-29. Próximo passo: MVP scope detalhado e plano de implementação.*
