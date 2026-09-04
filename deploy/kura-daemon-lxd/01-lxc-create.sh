#!/usr/bin/env bash
# Cria o container LXC kura-daemon (Ubuntu 24.04) e instala Docker dentro
# (nesting — os agentes gerenciados podem precisar de containers próprios).
# Não sobe nenhum stack aqui: este LXC roda só o `kurad`, nada de
# relay/postgres/etc (isso já existe em deploy/kura-dev/).
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-daemon}"
POOL="${KURA_LXC_POOL:-oute-pool}"

if lxc info "$CONTAINER" >/dev/null 2>&1; then
  echo "==> LXC '$CONTAINER' já existe — pulando criação."
else
  # `ubuntu:24.04` sem pin de arquitetura: o oute-server é arm64 e o LXD
  # resolve a imagem para a arquitetura do host automaticamente (mesmo
  # comportamento do deploy/kura-dev/01-lxc-create.sh).
  echo "==> Criando LXC '$CONTAINER' (Ubuntu 24.04, pool '$POOL')…"
  lxc launch ubuntu:24.04 "$CONTAINER" \
    -s "$POOL" \
    -c limits.cpu=2 \
    -c limits.memory=4GiB \
    -c security.nesting=true \
    -c security.syscalls.intercept.mknod=true \
    -c security.syscalls.intercept.setxattr=true
fi

echo "==> Aguardando rede do container…"
for _ in $(seq 1 30); do
  IP=$(lxc exec "$CONTAINER" -- hostname -I | tr " " "\n" | grep -m1 "^10\." || true)
  [ -n "$IP" ] && break
  sleep 2
done
[ -n "${IP:-}" ] || { echo "ERRO: container sem IP"; exit 1; }
echo "    IP do container: $IP"

echo "==> Instalando Docker no container (nested — para os agentes, não para o kurad)…"
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
  docker --version
'

echo
echo "OK. LXC '$CONTAINER' pronto em $IP. Próximo passo: ./02-toolchain.sh"
