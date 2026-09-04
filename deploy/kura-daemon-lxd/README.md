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
- **Gap de CI a resolver antes de `03-kurad.sh` funcionar de verdade:**
  o job `server-cross-compile` em `.github/workflows/ci.yml` compila
  `kura-acp`, `kura-agent`, `kura-dev-mcp`, `git-credential-nostr` e
  `git-sign-nostr` para `aarch64-unknown-linux-musl` via `cross@0.2.5`, mas
  **`kurad` não está nesse matrix**, e esse job hoje só compila/testa — não
  há passo de upload/publish de artefato para nenhum desses binários de
  servidor. Antes de rodar `03-kurad.sh` em produção é preciso: (1) adicionar
  `-p kurad` ao matrix de cross-compile, e (2) criar algum passo de
  release/publish (release do GitHub, GHCR, etc.) que deixe o binário arm64
  baixável por URL. Isso é trabalho de CI separado, fora deste PR — só
  documentado aqui.

## Estado atual do `kurad` (verificado em `crates/kurad/src/main.rs`)

- CLI: só `kurad run --data-dir <path> [--relay <url>] [--dev]` e
  `kurad status --data-dir <path>` (flags com equivalentes em env var:
  `KURA_DATA_DIR`, `KURA_RELAY_URL`).
- **Sem `config.toml`**: os parâmetros são só flag/env, não há leitura de
  arquivo de configuração. `03-kurad.sh` por isso não escreve nenhum
  `config.toml` — os parâmetros vão direto na unit systemd. Suporte a
  config.toml é coisa da Fase 4 (service-manager integration), não construída
  ainda.
- **Sem `kurad service install`**: `03-kurad.sh` escreve a unit systemd à mão
  como stopgap; quando a Fase 4 trouxer esse subcomando, a unit gerada por
  ele deve substituir a escrita à mão.
- **Sem `kurad identity lock/unlock`**: essa feature (passphrase NIP-49,
  `identity.ncryptsec`, `KURA_IDENTITY_PASSPHRASE`, ver plano D4) está em
  desenvolvimento paralelo, fora deste checkout — não existe neste binário
  hoje. `03-kurad.sh` documenta o passo como best-effort/forward-looking e a
  unit systemd já deixa uma linha `EnvironmentFile=` comentada, pronta para
  quando existir.
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
uma vez interativamente (comando best-effort, ver acima) e só então habilite:

```bash
lxc exec kura-daemon -- sudo -u kura kurad identity lock --data-dir /var/lib/kura
lxc exec kura-daemon -- systemctl enable --now kurad
```

## Secrets

Não há secrets pedidos interativamente nestes scripts (diferente do
`deploy/kura-dev/`, que pede senha do Postgres etc.) — o `kurad` de hoje
resolve identidade via keyring → `<data-dir>/identity.key` → gera e salva, e
não tem passphrase própria ainda. Quando `KURA_IDENTITY_PASSPHRASE` existir
(ver "Estado atual"), o operador deve colocá-la num arquivo root-only
(`chmod 600 /etc/kurad/identity.env` dentro do LXC) e descomentar a linha
`EnvironmentFile=` na unit — nunca em texto plano no script.

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
| `KURA_RELEASE_URL` | release do GitHub (ver comentário em `03-kurad.sh`) | `03-kurad.sh` |
