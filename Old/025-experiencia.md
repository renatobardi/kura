# Kura — Camada de Experiência (visual, UX, CX, personalidade)

> Extração da camada de experiência para reexportar/recriar em outro stack. Citações em `arquivo:linha`.
> Suposições marcadas com **[INFERIDO]**; ausências com **NÃO ENCONTRADO**. Base: `spec/02-funcional.md`.
> Norte estético declarado: "ChatGPT-clone (preto/cinza/verde-accent) com acentos editoriais Mincho"
> (`templates/design.css:11-13`). Frontend é JS vanilla sem build; CSS único em `templates/design.css` +
> CSS inline em `templates/app.html`. Personalidade é fortemente japonesa (kanji 蔵, daruma, koi, sakura, chá).

---

## 1. IDENTIDADE VISUAL (design tokens)

### 1.1 Paleta de cores — tema escuro (default)
Fonte: `templates/design.css:44-53` (`:root, html[data-theme="dark"]`).

| Token | Valor | Papel |
|---|---|---|
| `--bg` | `#212121` | fundo principal |
| `--sidebar` | `#171717` | barra lateral |
| `--surface` | `#2F2F2F` | cartões/superfícies |
| `--surface-hover` | `#3F3F3F` | superfície em hover |
| `--border` | `#2F2F2F` | borda padrão |
| `--border-strong` | `#3F3F3F` | borda forte |
| `--text` | `#ECECEC` | texto primário |
| `--text-2` | `#B4B4B4` | texto secundário |
| `--text-3` | `#8E8E8E` | texto terciário/placeholder |
| `--hover` | `rgba(255,255,255,0.06)` | overlay hover |
| `--active` | `rgba(255,255,255,0.1)` | overlay ativo |
| `--link` | `#7AB7FF` | links |
| `--accent` | `#10A37F` | accent (verde ChatGPT) |
| `--red` | `#EF4444` | erro/perigo |
| `--orange` | `#F59E0B` | aviso |
| `--green` | `#10B981` | sucesso |
| `--user-bubble` | `#2F2F2F` | balão do usuário |
| `--shadow-modal` | `0 24px 48px -8px rgba(0,0,0,0.5)` | sombra de modal |

### 1.2 Paleta de cores — tema claro
Fonte: `templates/design.css:54-63` (`html[data-theme="light"]`).

| Token | Valor |
|---|---|
| `--bg` | `#FFFFFF` |
| `--sidebar` | `#F9F9F9` |
| `--surface` | `#F4F4F4` |
| `--surface-hover` | `#ECECEC` |
| `--border` | `#E5E5E5` |
| `--border-strong` | `#D1D1D1` |
| `--text` | `#0D0D0D` |
| `--text-2` | `#5D5D5D` |
| `--text-3` | `#8E8E8E` |
| `--hover` | `rgba(0,0,0,0.05)` |
| `--active` | `rgba(0,0,0,0.08)` |
| `--link` | `#2F80ED` |
| `--accent` | `#10A37F` (igual ao dark) |
| `--red` | `#DC2626` |
| `--orange` | `#D97706` |
| `--green` | `#059669` |
| `--user-bubble` | `#F4F4F4` |
| `--shadow-modal` | `0 24px 48px -8px rgba(0,0,0,0.15)` |

> Cores sazonais (banner) e ripple/koi têm valores próprios — ver §6.

### 1.3 Tipografia
Fonte: `templates/design.css:72-108`.
- **Família única em TUDO** (títulos, body, código, kbd, marca): `--font` =
  `ui-sans-serif,-apple-system,BlinkMacSystemFont,"Segoe UI",Inter,"Helvetica Neue",Arial,"Hiragino Sans",
  "Yu Gothic","Noto Sans CJK JP",sans-serif`. (`design.css:81`). O fallback CJK no fim é proposital para
  renderizar o kanji 蔵 em qualquer SO. (`design.css:75-80`).
- **Aliases legados** apontam todos para o mesmo token: `--font-sans`, `--font-mono`, `--font-mincho`
  (`design.css:82-84`). Ou seja, apesar da menção a "Mincho", **não há fonte serifada real carregada**
  [INFERIDO da resolução dos aliases]; nenhuma `@font-face`/webfont própria — **NÃO ENCONTRADO** arquivo de
  fonte (ver §8).
- **Escala tipográfica (6 tamanhos + 2xs)** (`design.css:101-108`): `--fs-2xs:10px` · `--fs-xs:11px`
  (eyebrow/chip/badge) · `--fs-sm:12px` (help/meta) · `--fs-base:13.5px` (body/listas) · `--fs-md:14px`
  (input/botão/header chat) · `--fs-lg:16px` (título de seção) · `--fs-xl:24px` (título de página) ·
  `--fs-display:30px` (hero/welcome).
- **Line-heights** (`design.css:111-113`): `--lh-tight:1.3` · `--lh-normal:1.5` · `--lh-relaxed:1.6`.
- **Pesos observados**: 500 (botões/itens), 600 (marca/headers de modal), 700 (kanji da marca/favicon).
  (`app.html:29,595`, `design.css:444,623`).

### 1.4 Espaçamento, raios, sombras
- **Espaçamento** (grid de 4px, `design.css:118-124`): `--space-1:4px` … `--space-7:48px`
  (`1:4, 2:8, 3:12, 4:16, 5:24, 6:32, 7:48`).
- **Raios** (`design.css:127-132`): `--radius-sm:6px` · `--radius-md:8px` · `--radius-lg:12px` ·
  `--radius-xl:14px` · `--radius-pill:9999px` · `--radius-circle:50%`.
- **Sombras**: única sombra tokenizada é `--shadow-modal` (varia por tema, §1.1/1.2), reutilizada por modal e
  toast (`design.css:620,723`). Demais elevações usam borda, não sombra. [INFERIDO]

---

## 2. LAYOUT & COMPONENTES

### 2.1 Estrutura de tela (shell)
- **Documento**: `<html lang="pt-BR" data-theme="dark">` (`app.html:2`); SPA de página única.
- **Layout principal**: sidebar fixa à esquerda + área de conteúdo. Sidebar `width:260px`, colapsável para
  `60px` com transição `width 0.2s ease`; no modo colapsado, esconde labels/atalhos. (`app.html:21-24`).
- **Cabeçalho da sidebar** (`.sb-header`, altura 60px) com marca `.sb-brand` (kanji + "kura") e toggle
  `.sb-toggle`. (`app.html:26-30`).
- **Roteamento por hash** (SPA): views `new`, `setup`, `settings`, `settings/upgrade`, `dashboard`,
  `vault[/graph|/lint|/consolidate|/<path>]`, `pre-projects`, `pp/<id>`, `projects`, `p/<id>[/<tab>]`,
  `queue`, `c/<chatId>`; default = `new`. (`spa/04_state.js:660-704`).
- **Hierarquia visual**: página centrada com largura máxima de 820px e padding-top generoso
  (`.k-page`, ver design system); títulos via `.k-title` (`--fs-xl`), seções via `.k-heading` (`--fs-lg`),
  eyebrows via `.k-eyebrow` (`--fs-xs`). (`design.css:21`, seção 4-5).

### 2.2 Sistema de design `.k-*` (componentes reutilizáveis)
Fonte: `templates/design.css` (13 seções declaradas em `:1-34`). É a **fonte única** de classes de UI; um
teste falha se houver classe `.k-*` usada no JS sem definição no CSS (`design.css:32-34`;
`tests/test_design_system_completeness.py`).

- **Tipografia**: `.k-title`, `.k-heading`, `.k-eyebrow`.
- **Layout**: `.k-page` (container 820px), `.k-card`, `.k-toolbar`.
- **Formulário**: `.k-input`, `.k-textarea`, `.k-field` — foco muda só a cor da borda
  (`transition:border-color .15s`; foco → `--text-3`; sem outline). (`design.css:385-413`).
- **Ações** `.k-btn`: altura padrão 34px, `.k-btn-lg` 40px (`design.css:429-431`); variantes
  `primary`/`ghost`/`danger`; hover → `--surface-hover`. (`design.css:432-449`).
- **Feedback**: `.k-pill` (`info`/`neutral`), `.k-callout` (`error`/`success`/`warn`/`info`), `.k-empty`.
- **Navegação**: `.k-tab-strip`, `.k-tab`; colapsáveis com chevron que rotaciona 90° ao abrir
  (`design.css:337-343`).
- **Surface**: `.k-modal` (raio xl, sombra de modal), `.k-list`/`.k-list-row` (hover `--hover`),
  `.k-status-badge`.
- **F1**: `.k-toast` (stack top-right), tooltip CSS-only via `[data-tooltip]`, `.k-kbd`.
- **Específicos**: `.k-drop-zone` (área de drag-drop), `.k-radio-card` (`accent-color:var(--accent)`).
  (`design.css:1054-1079`).

### 2.3 Primitivos imperativos (JS)
Fonte: `templates/spa/02_primitives.js`.
- **`el(tag, attrs, ...children)`** — fábrica de DOM (helper base). (`02_primitives.js:8-24`).
- **`kModal/kConfirm/kAlert/kPrompt`** — diálogos como Promises; 1º botão recebe foco; primary destacado,
  danger em vermelho; ESC/clique no overlay = cancelar (quando dismissível). (`02_primitives.js:33-52`).
- **`kToast(message, {type,duration,closable})`** — notificação transitória; default `info`, 4000ms.
  (`02_primitives.js:211-242`).
- **`kEmpty({icon,title,help,action})`** — estado vazio (ícone pode ser id de sprite). (`02_primitives.js:244-272`).
- **`kSpinner({variant})`** — spinner (ver §3/§6).

### 2.4 Ícones (sprite SVG inline)
Sprite `<svg>` oculto com `<symbol>` em `app.html:587-622` (estilo Lucide/Feather, traço fino, `viewBox 0 0 24 24`).
Ids: `i-dashboard, i-search, i-plus, i-sun, i-moon, i-chevron, i-mask, i-paperclip, i-send, i-copy, i-trash,
i-edit, i-sparkles, i-dots, i-x, i-globe, i-panel, i-file, i-alert, i-ban, i-link, i-list, i-users, i-check,
i-book` (+ marca `kura-mark`). Ver §8.

---

## 3. INTERAÇÕES & MICROINTERAÇÕES

> Para cada uma: gatilho · duração · easing · o que comunica · arquivo.

- **Colapsar sidebar** — *gatilho*: clique no toggle · `width 0.2s ease` · comunica reconfiguração de espaço.
  (`app.html:21`).
- **Hover de botão `.k-btn`** — *gatilho*: hover · `transition:background .15s, border-color .15s, opacity .15s`
  · feedback de alvo clicável. (`design.css:446`).
- **Foco de input/textarea** — *gatilho*: foco · `transition:border-color .15s` (borda → `--text-3`, sem
  outline) · indica campo ativo. (`design.css:388,410`).
- **Chevron de colapsável** — *gatilho*: abrir seção · `transform:rotate(90deg)`, `transition .15s` · estado
  aberto/fechado. (`design.css:340-343`).
- **Tooltip CSS** — *gatilho*: hover/foco em `[data-tooltip]` · `opacity .12s ease-out` · dica contextual
  sem JS. (`design.css:767-783`).
- **Toast entra/sai** — *gatilho*: criar/dismiss · entra `k-toast-in .18s ease-out` (sobe 8px + fade), sai
  `k-toast-out .15s ease-in` · notificação não-bloqueante. (`design.css:730-745`).
- **Loader de digitação (3 pontos)** — `.typing .dot` com `@keyframes blink` (0%,80%,100% opacity .2 / 40%
  opacity 1), delays escalonados · estado "digitando". (`app.html:212-213`).
- **Loader "thinking" daruma (typewriter de verbos JP)** — *gatilho*: turno de chat aguardando o LLM ·
  digita verbo letra a letra a **85ms/char**, pausa **700ms**, apaga a **55ms/char**, gap **220ms**, escolhe
  novo verbo; cursor `|` pisca em `kura-cursor-blink .6s step-end infinite` · comunica "pensando e escrevendo".
  (`spa/02_primitives.js:345-424`; CSS `app.html:215-221`). ~50 verbos em `_KURA_VERBS`
  (`02_primitives.js:292-343`), ex.: 考えています (pensando), 調べています (pesquisando), 統合しています
  (integrando).
- **Spinner `kSpinner`** — quando eggs ativos, escolhe **aleatoriamente** entre `default/enso/water/sakura`;
  com eggs desligados, sempre `default`. (`spa/04_state.js:393-412`). Animações: `enso` gira 1.2s
  cubic-bezier(0.65,0,0.35,1) com largura de borda pulsando; `sakura` pétala que cai e gira 1.6s. (§6).
- **Drag-and-drop de anexo** — overlay "Drop to attach" sobre o composer; `.k-drop-zone` com hover
  `background:var(--surface-hover)`, `transition:border-color/.15s, background .15s`. (`design.css:1054-1060`;
  string `composer.drop_overlay`, `01_i18n.js:502`).
- **Abort de request ao navegar** — `apiAbortable()` cancela fetch pendente para evitar que resposta atrasada
  sobrescreva a nova view (anti-flicker). (`02_primitives.js:463-476`).
- **Refresh de título ao sair do chat** — navegar para fora dispara refresh; `beforeunload`/`pagehide` enviam
  beacon. (`spa/04_state.js:727-786`).
- **Mizu drop (ripple)** — ver §6 (microinteração que também é easter egg).

---

## 4. ESTADOS DE UI

- **Vazio (empty state)** — primitivo `.k-empty` / `kEmpty()` com ícone + título + ajuda + ação centrados,
  cor `--text-3`. (`design.css:521-542`, `02_primitives.js:244-272`). Exemplos de copy:
  `pre_projects.empty` = "No pre-projects yet. Create one to group chats and files around a topic.";
  `projects.empty` = "No projects yet…"; `queue.empty_inbox` = "Inbox is empty. Drop files into the folder
  above to process."; `vault.no_pages_to_show` = "No pages to show." (`01_i18n.js:126,207,291,416`). Variante
  zen (egg): `egg.zen.empty` = "Empty for now. Like a clean shoji screen — full of potential."
  (`01_i18n.js:408`).
- **Carregando** — chat usa o loader daruma (§3); listagens/vault usam `kSpinner` (§3/§6); textos como
  `dashboard.loading` = "Loading dashboard…", `chat.thinking` = "Thinking…". (`01_i18n.js:483,104`).
- **Erro** — banner `.k-callout-error` (fundo `rgba(239,68,68,0.10)`, cor `--red`) e/ou `kToast` tipo `error`.
  (`design.css:516,735`). Mensagens genéricas, prefixos reutilizáveis: `common.error_prefix` = "Error: ",
  `vault.search_err` = "Search error: ", `queue.error_generic` = "Queue request failed: ",
  `queue.error_busy` = "Queue is busy — another run is in progress." (`01_i18n.js:488,487,305,304`). O servidor
  não vaza stack (regra em `spec/02-funcional.md` §4).
- **Sucesso** — `.k-callout-success` (cor `--accent`) e/ou `kToast` tipo `success` (borda/ícone accent).
  (`design.css:517,738`).
- **Aviso** — `.k-callout-warn` / `kToast` `warn` (laranja). (`design.css:518,740`).
- **Offline** — tratamento explícito de estado offline (banner/retry dedicado): **NÃO ENCONTRADO**. O app é
  local-first (servidor no próprio host), então "offline de rede" não é um estado de primeira classe
  [INFERIDO]. Falhas de fetch caem nos callouts/toasts de erro acima.

---

## 5. COPY & TOM DE VOZ

- **Bilíngue EN + pt-BR** com paridade obrigatória de chaves (test `tests/test_i18n.py`; ~igual número de
  chaves nas duas tabelas em `spa/01_i18n.js`). Resolução de idioma:
  `window.KURA_LANG → localStorage.kuraLang → navigator.language → en`. [INFERIDO da estrutura de `01_i18n.js`]
- **Tom**: direto, conciso, técnico-amigável e levemente caloroso; sem gírias, com toques poéticos pontuais
  (sobretudo nos eggs/zen). Exemplos:
  - Onboarding: `wizard.title` = "Welcome to Kura"; `wizard.welcome.body` = "Kura is a local-first second
    brain. The next few steps will set up where your data lives…". (`01_i18n.js:25,33`).
  - Hero do chat: `chat.welcome_title` = "How can I help?". (`01_i18n.js:506`).
  - Placeholders: `composer.placeholder` = "Ask something"; `composer.placeholder_hint` = "Ask a question,
    attach a file, or paste a snippet. Answers come from Devin."; `search.placeholder` = "Search chats…";
    `vault.search_placeholder` = "Search the vault (FTS5)…". (`01_i18n.js:501,111,118,485).
  - Botões/ações: `composer.send` = "Send", `composer.attach` = "Attach file", `queue.process_all` =
    "Process all", `wizard.finish` = "Finish". (`01_i18n.js:503,504,294,31`).
  - Microcopy útil: `chat.duplicate_title_hint` = "Another chat already has this title — I'll append (2), (3)
    on save." (1ª pessoa, "I'll"). (`01_i18n.js:106`).
  - Subtítulos explicativos com jargão técnico assumido: `projects.subtitle` cita "mini-vault (SQLite +
    FTS5)"; `queue.subtitle` descreve o pipeline "dedups, parses, and ingests". (`01_i18n.js:206,289`).
- **Voz dos easter eggs (poética/zen, com kanji)**: `egg.cha_break.message` = "Cha break? 茶 Forty-five
  minutes in. A few minutes of tea couldn't hurt."; `egg.zen.greeting` = "月 the storehouse keeps your work
  safe through the night."; `egg.season.spring` = "Spring 春". (`01_i18n.js:405-406,401`). Equivalentes
  pt-BR existem (ex.: "Cha break? 茶 Quarenta e cinco minutos sem pausa…"). (`01_i18n.js:1014-1016`).
- **Placeholders de exemplo personalizados ao usuário-alvo** (Santander/risk): `projects.name_placeholder` =
  "e.g. Risk Engines Q4"; `pre_projects.name_placeholder` = "e.g. Tribe planning". (`01_i18n.js:211,130`).

---

## 6. EASTER EGGS & DETALHES ESCONDIDOS

> Investigação ativa (grep por `easter|secret|konami|confetti|petal|sakura|koi|daruma|kanji|mizu|cha-break|
> enso|moon|season`). Há um **subsistema dedicado** "plano E (E1-E7)". **Nenhum é código morto** — todos têm
> handler ativo registrado.

**Infraestrutura de controle:**
- **Focus mode (master kill-switch)** — `html[data-focus-mode="on"] .kura-egg{display:none !important}`
  esconde TODA decoração; eggs JS verificam `kuraEggsEnabled()`. (`design.css:808-817`).
- **Toggle por egg** — Settings → Easter eggs grava `localStorage.kuraEggsPrefs`; default = tudo ligado;
  `body[data-eggs-disabled~="<id>"]` esconde os always-on via CSS, eggs JS checam `kuraEggEnabled(id)`.
  (`spa/04_state.js:281-336`; `design.css:819-828`). Ids: `season-banner, kanji-cycle, daruma-blink,
  sakura-petals, koi-konami, koi-7x-click, mizu-drop, cha-break, moon` (`04_state.js:281-291`).

**Eggs (como dispara · o que acontece · arquivo:linha):**
- **E2 — Ciclo de kanji** — *clicar a marca* `.kura-mark` · cicla o glyph 蔵→庫→倉→記→心→知→書 (atualiza o
  `<text>` do `<symbol>`); reseta no reload (não persiste). (`spa/04_state.js:351-362, 380-384`).
- **E2 — Daruma blink** — *automático*, intervalo aleatório **35–90s** · a marca "pisca" (anim
  `kura-blink-once` 700ms, `scaleY`). (`spa/04_state.js:364-376`; CSS `design.css:830-841`).
- **E3 — Banner sazonal** — *no boot* · faixa fixa de 4px no topo, gradiente por estação (primavera rosa,
  verão azul, outono vermelho/laranja, inverno azul-claro/cinza), `opacity:0.55`, `aria-hidden`.
  (`spa/04_state.js:414-435`; CSS `design.css:907-918`). Estação por mês (hemisfério norte): mar-mai
  primavera, jun-ago verão, set-nov outono, resto inverno. (`04_state.js:414-420`).
- **E3 — Variantes de spinner** — *ao mostrar um spinner com eggs ativos* · escolhe aleatório
  `enso/water/sakura` em vez do `default`. (`04_state.js:393-412`; CSS `design.css:858-905`).
- **E4 — Código Konami** — *sequência* `↑ ↑ ↓ ↓ ← → ← → b a` (ignorada dentro de inputs/textarea/
  contenteditable) · um **koi** (SVG laranja com gradiente + barbatana + olho) nada da esquerda para a
  direita ~5s e se remove. (`spa/04_state.js:446-470, 486-511, 513`; CSS koi `design.css:920-938`).
- **E4 — Koi por 7 cliques** — *clicar o label "kura"* `.sb-brand .sb-label` **7× em <5s** · invoca o mesmo
  `swimKoi()`. (`spa/04_state.js:472-484, 520-523`).
- **E5 — Pétalas de sakura** — *no boot*, por **15s** (`_burstSakuraPetals(15000)`) · pétalas caem
  (`kura-petal-fall`, queda 7–12s, rotação); na **primavera** a cadência dobra (intervalo 1000ms vs 2000ms).
  (`spa/04_state.js:524, 549-573`; CSS `design.css:940-956`). Comentário registra que havia uma lanterna
  chochin removida a pedido do usuário (`04_state.js:544-547`). **NÃO ENCONTRADO** no código atual (só
  comentário).
- **E6 — Mizu drop (ripple)** — *clicar qualquer `.k-btn-primary`* · anel concêntrico expande e some
  (`kura-mizu-ripple 0.7s ease-out`, escala 0→8); aparece no ponto do clique; removido após 800ms.
  (`spa/04_state.js:579-597`; CSS `design.css:958-972`).
- **E6 — Cha break (pausa para chá)** — *após ~45min de sessão* (`_CHA_BREAK_MS=45*60*1000`), checado a cada
  5min · toast lembrando de fazer uma pausa para chá; só dispara se houve atividade nos últimos 15min e não
  avisou na última 1h; ociosidade reseta o timer (atividade = keydown/click/hashchange).
  (`spa/04_state.js:599-647`). Copy: `egg.cha_break.message` (com 茶). (`01_i18n.js:405`).
- **E7 — Lua noturna** — *no boot*, se a hora local for **22:00–05:59** · lua decorativa no canto superior
  direito, `title` = `egg.zen.greeting` (com 月). (`spa/04_state.js:614-626`; CSS `design.css:991-994`).
- **E7 — Hanko (selo) `.kura-hanko`** — elemento decorativo com `::before{content:'蔵'}`. (`design.css:974-989`).
  Gatilho de inserção dinâmica: **NÃO ENCONTRADO** explicitamente nos arquivos lidos (classe definida; uso
  pode estar em `05/06_views`). [INFERIDO]
- **Favicon temático** — SVG inline data-URI do kanji 蔵, que **adapta a cor ao tema do SO** via
  `@media (prefers-color-scheme: dark)` embutido no próprio SVG. (`app.html:7`).

> Mensagens no console (console.log secretos), confetti, ou gatilhos por datas específicas (ex.: aniversário):
> **NÃO ENCONTRADO** além das estações/horário noturno acima.

---

## 7. ACESSIBILIDADE & RESPONSIVIDADE

- **Temas**: dark/light/system via `html[data-theme]` (`app.html:2`); tokens trocam por seletor de tema
  (`design.css:44-63`). Preferência em `localStorage.kuraTheme` [INFERIDO do padrão de toggles]; sincronização
  cross-aba via evento `storage` [INFERIDO]. Ícones de toggle `i-sun`/`i-moon` (`app.html:600-601`).
- **prefers-reduced-motion**: respeitado ao menos no cursor do loader — `@media (prefers-reduced-motion:reduce)
  {.kura-eater-cursor{animation:none;opacity:1}}`. (`app.html:222`). Cobertura completa das demais animações
  sob reduced-motion: **NÃO ENCONTRADO** (parcial).
- **Breakpoints (CSS `@media`)**: `max-width:1100px` esconde `.vault-meta`; `max-width:880px` reduz
  `.vault-tree` para 200px. (`app.html:437-442`). Cobertura mobile ampla/global: **limitada** — não há um
  sistema de breakpoints global; o layout é desktop-first. [INFERIDO]
- **Suporte a teclado**: diálogos fecham com ESC; 1º botão recebe foco (`02_primitives.js:33-44`); tooltips
  aparecem em `:focus-visible` (`design.css:783`). Konami ignora eventos em campos de texto
  (`04_state.js:457-459`). Atalho de busca global (⌘K): referenciado como `.sb-shortcut` [INFERIDO] —
  binding exato **NÃO ENCONTRADO** nos trechos lidos.
- **ARIA**: loader daruma usa `role="status"` + `aria-live="polite"` + `aria-label` (`02_primitives.js:353-357`);
  spinner tem `aria-label="loading"` (`04_state.js:409`); todas as decorações/eggs são `aria-hidden="true"`
  (`04_state.js:431,493,568,583,623`); sprite SVG de ícones é `aria-hidden="true"` (`app.html:587`).
- **Idioma do documento**: `lang="pt-BR"` fixo no HTML servido (`app.html:2`), embora a UI seja bilíngue
  [INFERIDO — o atributo não é atualizado dinamicamente nos trechos lidos].

---

## 8. ASSETS

> Para reexportar/recriar no novo stack.

- **Fonte**: nenhuma webfont/arquivo de fonte próprio — usa **apenas fontes do sistema** via stack
  `--font` (`design.css:81`). Reexportação: replicar o stack; garantir fallback CJK (`Hiragino Sans`/
  `Yu Gothic`/`Noto Sans CJK JP`) para o kanji 蔵.
- **Ícones (sprite SVG inline)**: 25+ símbolos em `app.html:587-622` (estilo traço, `viewBox 0 0 24 24`):
  `i-dashboard, i-search, i-plus, i-sun, i-moon, i-chevron, i-mask, i-paperclip, i-send, i-copy, i-trash,
  i-edit, i-sparkles, i-dots, i-x, i-globe, i-panel, i-file, i-alert, i-ban, i-link, i-list, i-users,
  i-check, i-book` (+ `i-refresh` referenciado em strings, `01_i18n.js:491`). **NÃO ENCONTRADO** arquivos
  `.svg` separados — todos inline no HTML.
- **Marca / logotipo**: kanji **蔵** como `<symbol id="kura-mark">` (texto em `currentColor`, adapta ao tema)
  (`app.html:592-596`). Sem arquivo de imagem de logo.
- **Favicon**: SVG data-URI inline (kanji 蔵, adapta dark/light) (`app.html:7`). Sem `.ico`/`.png`.
- **Ilustrações (SVG geradas em runtime)**:
  - **Koi** — SVG inline construído em JS (corpo laranja com gradiente `koig`, barbatana `#B5302A`, olho
    `#1C1C1C`). (`spa/04_state.js:497-508`).
  - **Pétala / enso / sakura / mizu / season banner / moon / hanko** — puramente CSS (gradientes, border-radius,
    keyframes), sem assets externos. (`design.css:858-994`).
- **Cores de assets decorativos** (para recriação fiel): pétala `rgba(255,182,193,0.85)` (`design.css:946`);
  estações `spring #F8C8D8/#FBD3E1`, `summer #3A5F8F/#9FCFFF`, `autumn #B5302A/#D97706`,
  `winter #9FCFFF/#EDEDED` (`design.css:915-918`); koi `#F8C8B0→#E27B4C`, barbatana `#B5302A`
  (`04_state.js:500-505`).
- **Sons / áudio**: **NÃO ENCONTRADO**.
- **Lottie / JSON de animação**: **NÃO ENCONTRADO** (todas as animações são CSS keyframes ou typewriter JS).
- **Imagens raster (PNG/JPG/GIF/WebP)**: **NÃO ENCONTRADO** em `templates/` (UI é 100% vetorial/CSS).

---

## Resumo de [INFERIDO] / NÃO ENCONTRADO
- **[INFERIDO]**: persistência/sync de tema, resolução de idioma, mobile/desktop desktop-first, uso dinâmico
  de `.kura-hanko`, ausência de fonte serifada real apesar do alias "mincho".
- **NÃO ENCONTRADO**: estado offline de 1ª classe; cobertura completa de reduced-motion; binding exato do ⌘K;
  console eggs/confetti/datas especiais (além de estações + lua noturna); arquivos de fonte, ícone, imagem
  raster, som ou Lottie separados (tudo é inline/CSS/sistema).
