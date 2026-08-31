#!/usr/bin/env bash
# Cria o container LXC kura-dev (Ubuntu 24.04) e instala Docker dentro.
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-dev}"

if lxc info "$CONTAINER" >/dev/null 2>&1; then
  echo "==> LXC '$CONTAINER' já existe — pulando criação."
else
  echo "==> Criando LXC '$CONTAINER' (Ubuntu 24.04)…"
  lxc launch ubuntu:24.04 "$CONTAINER" \
    -c limits.cpu=2 \
    -c limits.memory=4GiB \
    -c security.nesting=true \
    -c security.syscalls.intercept.mknod=true \
    -c security.syscalls.intercept.setxattr=true
fi

echo "==> Aguardando rede do container…"
for _ in $(seq 1 30); do
  IP=$(lxc list "$CONTAINER" -c 4 --format csv | awk '{print $1}' | head -1)
  [ -n "$IP" ] && break
  sleep 2
done
[ -n "${IP:-}" ] || { echo "ERRO: container sem IP"; exit 1; }
echo "    IP do container: $IP"

echo "==> Instalando Docker no container…"
lxc exec "$CONTAINER" -- bash -euo pipefail -c '
  export DEBIAN_FRONTEND=noninteractive
  if ! command -v docker >/dev/null 2>&1; then
    apt-get update -qq
    apt-get install -y -qq ca-certificates curl
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu noble stable" \
      > /etc/apt/sources.list.d/docker.list
    apt-get update -qq
    apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin
  fi
  docker --version && docker compose version
'

echo
echo "OK. LXC '$CONTAINER' pronto em $IP. Próximo passo: ./02-kura-up.sh"
