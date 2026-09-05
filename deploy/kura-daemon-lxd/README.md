# Kura — `kurad` no oute-server (LXD/LXC)

Sobe o `kurad` (daemon headless que roda os agentes gerenciados do Kura, sem
UI — ver `crates/kurad/src/main.rs`) num container LXC dedicado no
oute-server, como serviço systemd, para os agentes ficarem rodando 24/7 sem
depender de um Mac ligado. É o mesmo padrão de `deploy/kura-dev/` (que sobe o
relay via Docker Compose), adaptado para um binário Rust simples.

Este primeiro `kurad` aponta para o relay `kura-dev` já existente
(`wss://dev.kura.oute.pro`, decisão D9 do plano) — não sobe um relay novo.

**Este LXC só hospeda agentes.** Ele não serve console/UI nenhum: o `kurad`
de hoje não abre porta HTTP (ver `04-console-access.sh` e a seção "Estado
atual" abaixo). O desktop app não aponta para este LXC — ele continua se
conectando a relés Nostr normalmente; este daemon só mantém agentes vivos do
lado do relay `dev.kura.oute.pro`.

## Pré-requisitos

- LXD funcionando no oute-server (Oracle Cloud/Ampere, arm64), pool de
  storage `oute-pool`.
- Tailscale ativo na rede (usado por outros vhosts do host; não usado por
  este deploy hoje — ver "Estado atual").
- `deploy/kura-dev/` já rodando (`wss://dev.kura.oute.pro` no ar), já que é
  o relay que este `kurad` usa.
- **Binário `kurad` publicado**: o job `server-cross-compile` em
  `.github/workflows/ci.yml` agora inclui `-p kurad` no matrix (junto com
  `kura-acp`, `kura-agent`, `kura-dev-mcp`, `git-credential-nostr` e
  `git-sign-nostr`) para `x86_64-`/`aarch64-unknown-linux-musl` via
  `cross@0.2.5` — mas esse job só compila/testa em CI, nunca publica nada.
  O artefato baixável vem de um workflow separado,
  `.github/workflows/kurad-release.yml`: `workflow_dispatch` manual (sem
  trigger automático de push/tag — é um binário interno de ops, não um
  produto com cadência de release própria ainda), publica os dois binários
  numa release **rolling** de tag fixa `kurad-latest` (não usa
  `releases/latest`, que é compartilhado com as releases do desktop/mobile e
  mudaria de alvo sozinho). **Rode esse workflow manualmente (Actions →
  "kurad release" → Run workflow) pelo menos uma vez antes do `03-kurad.sh`**,
  e de novo sempre que quiser atualizar o binário no LXC.

## Estado atual do `kurad` (verificado em `crates/kurad/src/main.rs`)

- CLI: só `kurad run --data-dir <path> [--relay <url>] [--dev]
  [--reconcile-interval-secs <n>]` e `kurad status --data-dir <path>` (flags
  com equivalentes em env var: `KURA_DATA_DIR`, `KURA_RELAY_URL`,
  `KURA_RECONCILE_INTERVAL_SECS`).
- **Reconciliação periódica em runtime**: `run` não só restaura os agentes
  configurados na subida — a cada `--reconcile-interval-secs` (default 30s)
  ele reprova o relay do workspace e sobe o que deveria estar rodando e não
  está (`reconcile_managed_agent_runtimes`, a mesma função que o desktop já
  usava por comunidade). Isso fecha o gap que existia até aqui: adicionar um
  agente em `managed-agents.json` (ou marcar `start_on_app_launch`) enquanto
  o `kurad` já estava de pé exigia reiniciar o processo pra ele ser
  percebido. `--reconcile-interval-secs 0` desliga e volta ao comportamento
  antigo (só a restauração da subida).
- **Sem `config.toml`**: os parâmetros são só flag/env, não há leitura de
  arquivo de configuração. `03-kurad.sh` por isso não escreve nenhum
  `config.toml` — os parâmetros vão direto na unit systemd. Suporte a
  config.toml é coisa da Fase 4 (service-manager integration), não construída
  ainda.
- **Sem `kurad service install`**: `03-kurad.sh` escreve a unit systemd à mão
  como stopgap; quando a Fase 4 trouxer esse subcomando, a unit gerada por
  ele deve substituir a escrita à mão.
- **`kurad identity lock/unlock` existe** (D4, PR #14, `crates/kura-host/src/identity_lock.rs`
  + `crates/kurad/src/main.rs`): `kurad identity lock --data-dir <path>`
  cifra a identidade atual em `identity.ncryptsec` (NIP-49) sob uma
  passphrase; `kurad identity unlock --data-dir <path> [--remember]` verifica
  a passphrase e, com `--remember`, grava `identity.autounlock` (0600) para
  boot desatendido; `KURA_IDENTITY_PASSPHRASE` é o equivalente por env var
  (o que a unit systemd usa via `EnvironmentFile=`, ver "Secrets" abaixo).
  Sem nenhuma dessas três fontes de passphrase, `kurad run`/`status` numa
  identidade locked falham alto (não caem para uma chave efêmera).
- **Sem HTTP/API**: o próprio comentário de módulo do `main.rs` lista como
  "deliberately absent (later phases)": a API JSON-RPC/WebSocket, o web
  console, qualquer listener HTTP e qualquer integração de service-manager.
  Por isso `04-console-access.sh` é um placeholder — não há porta nenhuma
  para o `tailscale serve` apontar ainda.

## Ordem de execução (no oute-server, como usuário com lxc/sudo)

```bash
./01-lxc-create.sh        # cria o LXC kura-daemon (Ubuntu 24.04, pool oute-pool) e instala Docker (nested)
./02-toolchain.sh          # node 24, git, rg, gh, uv, claude/codex/goose CLIs, Playwright+Chromium
./03-kurad.sh              # baixa kurad, cria usuário 'kura', escreve a unit systemd (não habilita ainda)
./04-console-access.sh     # placeholder — kurad não tem HTTP/API hoje, ver acima
./05-validate.sh           # LXC, docker, toolchain, binário, `kurad status`, unit systemd
```

Depois do `03-kurad.sh`, antes de habilitar o serviço, configure a identidade
uma vez interativamente e só então habilite:

```bash
lxc exec kura-daemon -- sudo -u kura kurad identity lock --data-dir /var/lib/kura
```

Isso pede uma passphrase (duas vezes, para confirmar) e cifra a identidade
atual (a que o `kurad run` já teria resolvido via keyring →
`<data-dir>/identity.key` → gera-e-salva, sem `lock` nenhum) em
`identity.ncryptsec`. Sem uma unidade sistemd rodando de forma desatendida
capaz de fornecer essa passphrase, o `kurad` não vai destravar sozinho —
escolha uma das duas rotas antes de habilitar o serviço:

```bash
# Rota A — reinício desatendido sem prompt (equivalente em ameaça ao
# identity.key em texto plano de antes; ver "Secrets" abaixo):
lxc exec kura-daemon -- sudo -u kura kurad identity unlock --data-dir /var/lib/kura --remember

# Rota B — passphrase num arquivo root-only lido pela unit systemd (ver
# "Secrets" abaixo para como preparar /etc/kurad/identity.env):
lxc exec kura-daemon -- systemctl edit kurad --full   # descomente o EnvironmentFile=

lxc exec kura-daemon -- systemctl enable --now kurad
```

## Secrets

Não há secrets pedidos interativamente nestes scripts (diferente do
`deploy/kura-dev/`, que pede senha do Postgres etc.). A identidade do
`kurad` pode ficar em texto plano (`<data-dir>/identity.key`, comportamento
default — nenhuma ação extra) ou "locked" sob passphrase NIP-49 (rota
recomendada, ver seção acima). Para a rota B (passphrase via arquivo em vez
de `--remember`), a passphrase vai num arquivo root-only dentro do LXC —
nunca em texto plano no script nem commitada:

```bash
lxc exec kura-daemon -- bash -c 'install -d -m 0700 /etc/kurad && install -m 0600 /dev/stdin /etc/kurad/identity.env' <<'EOF'
KURA_IDENTITY_PASSPHRASE=<passphrase aqui>
EOF
lxc exec kura-daemon -- systemctl daemon-reload
```

depois descomente a linha `EnvironmentFile=-/etc/kurad/identity.env` na unit
(`03-kurad.sh` já a escreve comentada) antes de `systemctl enable --now kurad`.

## Operação

```bash
lxc exec kura-daemon -- systemctl status kurad
lxc exec kura-daemon -- journalctl -u kurad -f
lxc exec kura-daemon -- systemctl restart kurad
lxc exec kura-daemon -- sudo -u kura kurad status --data-dir /var/lib/kura
```

## Variáveis de ambiente (override)

| Variável | Default | Usada em |
|---|---|---|
| `KURA_LXC_NAME` | `kura-daemon` | todos |
| `KURA_LXC_POOL` | `oute-pool` | `01-lxc-create.sh` |
| `KURA_NODE_MAJOR` | `24` | `02-toolchain.sh` |
| `KURA_RELAY_DEV_URL` | `wss://dev.kura.oute.pro` | `03-kurad.sh` |
| `KURA_DAEMON_DATA_DIR` | `/var/lib/kura` | `03-kurad.sh`, `05-validate.sh` |
| `KURA_RELEASE_URL` | `.../releases/download/kurad-latest/kurad-aarch64-unknown-linux-musl` (publicado por `kurad-release.yml`) | `03-kurad.sh` |
