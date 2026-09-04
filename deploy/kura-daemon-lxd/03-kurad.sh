#!/usr/bin/env bash
# Instala o binário `kurad` dentro do LXC, cria o usuário de sistema `kura`,
# o data-dir e a unit systemd. NÃO habilita/inicia o serviço — isso fica para
# o operador rodar manualmente depois de configurar a identidade (ver
# mensagem final).
#
# Estado verificado neste checkout (crates/kurad/src/main.rs) em 2026-09:
#   - CLI é só `kurad run --data-dir <path> [--relay <url>] [--dev]` e
#     `kurad status --data-dir <path>`. Não existe `kurad service install`
#     (Fase 4 do plano, ainda não construída) nem `kurad identity lock/unlock`
#     (feature em desenvolvimento paralelo, fora deste checkout).
#   - `--data-dir` também aceita a env var KURA_DATA_DIR; `--relay` aceita
#     KURA_RELAY_URL. Não há suporte a config.toml — é só flag/env. Por isso
#     este script NÃO escreve um config.toml (escreveria um arquivo que o
#     binário não lê); os parâmetros vão direto na unit systemd.
#   - O plano (Fase 5, item 3) menciona `data-dir=/var/lib/kura` E um nest em
#     `/home/kura/.kura` — são a mesma ideia descrita de duas formas. Este
#     script usa só `/var/lib/kura` (via --data-dir), de forma consistente
#     com o usuário de sistema `kura` criado abaixo e com o que 05-validate.sh
#     espera.
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-daemon}"
RELAY_URL="${KURA_RELAY_DEV_URL:-wss://dev.kura.oute.pro}"
DATA_DIR="${KURA_DAEMON_DATA_DIR:-/var/lib/kura}"

# Ainda não existe workflow de release/publish para `kurad` em arm64: o job
# "Server Cross-Compile" do CI (.github/workflows/ci.yml) compila kura-acp,
# kura-agent, kura-dev-mcp, git-credential-nostr e git-sign-nostr para
# aarch64-unknown-linux-musl, mas NÃO inclui `kurad` no matrix ainda, e não há
# passo de upload/publish de artefato para nenhum desses binários de servidor
# hoje (o job só compila/testa). Ou seja: este script pressupõe uma URL que
# ainda não existe até esse trabalho de CI ser feito (ver README.md,
# "Pré-requisitos"). Ajuste KURA_RELEASE_URL quando esse artefato existir.
RELEASE_URL="${KURA_RELEASE_URL:-https://github.com/renatobardi/kura/releases/latest/download/kurad-aarch64-unknown-linux-musl}"

lxc info "$CONTAINER" >/dev/null || { echo "ERRO: LXC '$CONTAINER' não existe (rode 01-lxc-create.sh)"; exit 1; }

echo "==> Checando se o binário está disponível em: $RELEASE_URL"
if ! curl -fsI "$RELEASE_URL" >/dev/null 2>&1; then
  cat <<EOF
ERRO: não consegui alcançar $RELEASE_URL (HEAD falhou).

Isso é esperado até o job "Server Cross-Compile" do CI incluir \`kurad\` no
matrix e publicar o artefato arm64 em algum lugar baixável (release do
GitHub, GHCR, etc — ver README.md "Pré-requisitos"). Depois que existir,
exporte KURA_RELEASE_URL apontando para a URL real e rode de novo:

  KURA_RELEASE_URL=https://.../kurad-aarch64-unknown-linux-musl ./03-kurad.sh
EOF
  exit 1
fi

echo "==> Criando usuário de sistema 'kura' e data-dir ($DATA_DIR)…"
lxc exec "$CONTAINER" -- bash -euo pipefail -c "
  id kura >/dev/null 2>&1 || useradd --system --create-home --home-dir /home/kura --shell /usr/sbin/nologin kura
  install -d -o kura -g kura -m 0750 '$DATA_DIR'
"

echo "==> Baixando kurad…"
lxc exec "$CONTAINER" -- bash -euo pipefail -c "
  curl -fsSL '$RELEASE_URL' -o /usr/local/bin/kurad
  chmod 0755 /usr/local/bin/kurad
  /usr/local/bin/kurad --version || true
"

echo "==> Escrevendo unit systemd /etc/systemd/system/kurad.service…"
# Unit escrita à mão como stopgap: quando a Fase 4 trouxer
# `kurad service install --system`, esta unit deve ser substituída pela
# gerada pelo próprio binário.
lxc exec "$CONTAINER" -- bash -c "cat > /etc/systemd/system/kurad.service" <<EOF
[Unit]
Description=Kura headless daemon (kurad)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=kura
Group=kura
Environment=KURA_DATA_DIR=$DATA_DIR
Environment=KURA_RELAY_URL=$RELAY_URL
# Passphrase para desbloquear a identidade NIP-49 sem interação (feature
# ainda em desenvolvimento paralelo — ver README.md). Descomente e aponte
# para um arquivo root-only (chmod 600) quando "kurad identity lock/unlock"
# e KURA_IDENTITY_PASSPHRASE existirem neste binário.
# EnvironmentFile=-/etc/kurad/identity.env
ExecStart=/usr/local/bin/kurad run --data-dir $DATA_DIR --relay $RELAY_URL
Restart=on-failure
RestartSec=5s
NoNewPrivileges=true
ProtectSystem=strict
ReadWritePaths=$DATA_DIR
ProtectHome=true

[Install]
WantedBy=multi-user.target
EOF

echo "==> daemon-reload…"
lxc exec "$CONTAINER" -- systemctl daemon-reload

cat <<EOF

OK. kurad instalado em /usr/local/bin/kurad, data-dir $DATA_DIR, unit
kurad.service escrita mas NÃO habilitada ainda.

Antes de iniciar o serviço, configure a identidade uma vez, interativamente
(comando best-effort — só existirá quando a feature de identity lock
mencionada no plano, D4, chegar neste binário):

  lxc exec $CONTAINER -- sudo -u kura kurad identity lock --data-dir $DATA_DIR

Depois disso, habilite e inicie o serviço:

  lxc exec $CONTAINER -- systemctl enable --now kurad

Próximo passo (placeholder, ver 04-console-access.sh): ./04-console-access.sh
Depois: ./05-validate.sh
EOF
