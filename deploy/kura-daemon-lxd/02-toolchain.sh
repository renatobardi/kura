#!/usr/bin/env bash
# Instala dentro do LXC o toolchain base que os agentes gerenciados esperam
# (D10 do plano): node 24, git, ripgrep, gh, uv, claude/codex/goose CLIs e
# Playwright/Chromium. Docker já foi instalado (nested) pelo 01-lxc-create.sh.
#
# A lista de apps fica em /opt/kura-daemon/apps.toml dentro do LXC — um
# `[[app]]` por ferramenta, com o comando usado para checar presença. Não há,
# hoje, um formato apps.toml compartilhado com o `kurad`/`kura-host` (a
# checagem de presença de harness deles vive em
# crates/kura-host/src/managed_agents/readiness.rs e roda em Rust, não lê
# TOML) — este arquivo é só para os scripts de deploy, no espírito do
# `Requirement::MissingBinary` de lá: um app ausente é reportado, não instala
# nada por engano.
#
# Cada passo de instalação é idempotente (pula se o binário já existe).
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-daemon}"
NODE_MAJOR="${KURA_NODE_MAJOR:-24}"

lxc info "$CONTAINER" >/dev/null || { echo "ERRO: LXC '$CONTAINER' não existe (rode 01-lxc-create.sh)"; exit 1; }

echo "==> Escrevendo /opt/kura-daemon/apps.toml no container…"
lxc exec "$CONTAINER" -- mkdir -p /opt/kura-daemon
lxc exec "$CONTAINER" -- bash -c 'cat > /opt/kura-daemon/apps.toml' <<'EOF'
# Toolchain base esperado pelos agentes gerenciados do kurad (plano Fase 5,
# decisão D10). Gerado por deploy/kura-daemon-lxd/02-toolchain.sh — editar lá,
# não aqui.

[[app]]
name = "node"
check_command = "node --version"

[[app]]
name = "git"
check_command = "git --version"

[[app]]
name = "ripgrep"
check_command = "rg --version"

[[app]]
name = "gh"
check_command = "gh --version"

[[app]]
name = "uv"
check_command = "uv --version"

[[app]]
name = "claude"
check_command = "claude --version"

[[app]]
name = "codex"
check_command = "codex --version"

[[app]]
name = "goose"
check_command = "goose --version"

[[app]]
name = "playwright-chromium"
check_command = "npx --yes playwright --version"
EOF

echo "==> Instalando toolchain no container (idempotente)…"
lxc exec "$CONTAINER" -- env NODE_MAJOR="$NODE_MAJOR" bash -euo pipefail -c '
  export DEBIAN_FRONTEND=noninteractive

  if ! command -v node >/dev/null 2>&1 || [ "$(node -v | cut -dv -f2 | cut -d. -f1)" -lt "$NODE_MAJOR" ]; then
    echo "  -> node ${NODE_MAJOR}.x"
    curl -fsSL "https://deb.nodesource.com/setup_${NODE_MAJOR}.x" | bash -
    apt-get install -y -qq nodejs
  fi

  echo "  -> git, ripgrep, bzip2"
  apt-get update -qq
  apt-get install -y -qq git ripgrep bzip2

  if ! command -v gh >/dev/null 2>&1; then
    echo "  -> gh (GitHub CLI)"
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg -o /etc/apt/keyrings/githubcli-archive-keyring.gpg
    chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
      > /etc/apt/sources.list.d/github-cli.list
    apt-get update -qq
    apt-get install -y -qq gh
  fi

  if ! command -v uv >/dev/null 2>&1; then
    echo "  -> uv"
    curl -fsSL https://astral.sh/uv/install.sh | env UV_INSTALL_DIR=/usr/local/bin sh
  fi

  if ! command -v claude >/dev/null 2>&1; then
    echo "  -> claude CLI"
    npm install -g @anthropic-ai/claude-code
  fi

  if ! command -v codex >/dev/null 2>&1; then
    echo "  -> codex CLI"
    npm install -g @openai/codex
  fi

  if ! command -v goose >/dev/null 2>&1; then
    echo "  -> goose CLI"
    # O instalador baixa um .tar.bz2 e usa `tar` pra extrair, que por sua vez
    # invoca `bzip2` (instalado acima) — sem isso a extração falha em uma
    # instalação limpa do Ubuntu 24.04.
    curl -fsSL https://github.com/block/goose/releases/download/stable/download_cli.sh | CONFIGURE=false bash
  fi

  echo "  -> Playwright + Chromium"
  npx --yes playwright install --with-deps chromium
'

echo
echo "==> Checando presença de cada app (apps.toml)…"
MISSING=0
while IFS= read -r line; do
  case "$line" in
    name\ =\ *) APP_NAME=$(echo "$line" | sed -E 's/name = "(.*)"/\1/') ;;
    check_command\ =\ *)
      CMD=$(echo "$line" | sed -E 's/check_command = "(.*)"/\1/')
      if lxc exec "$CONTAINER" -- bash -lc "$CMD" >/dev/null 2>&1; then
        echo "  OK       $APP_NAME"
      else
        echo "  MISSING  $APP_NAME  (comando: $CMD)"
        MISSING=1
      fi
      ;;
  esac
done < <(lxc exec "$CONTAINER" -- cat /opt/kura-daemon/apps.toml)

echo
if [ "$MISSING" = 0 ]; then
  echo "OK. Toolchain completo. Próximo passo: ./03-kurad.sh"
else
  echo "ATENÇÃO: pelo menos um app está MISSING acima — revise antes de seguir."
  exit 1
fi
