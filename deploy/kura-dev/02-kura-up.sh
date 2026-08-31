#!/usr/bin/env bash
# Pergunta os secrets, escreve /opt/kura/.env e docker-compose.yml no LXC
# e sobe o stack (relay + postgres + redis + minio). Idempotente.
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-dev}"
DOMAIN="${KURA_DEV_DOMAIN:-dev.kura.oute.pro}"
IMAGE="${KURA_IMAGE:-ghcr.io/renatobardi/kura:main}"

lxc info "$CONTAINER" >/dev/null || { echo "ERRO: LXC '$CONTAINER' não existe (rode 01-lxc-create.sh)"; exit 1; }

if lxc exec "$CONTAINER" -- test -f /opt/kura/.env; then
  echo "==> /opt/kura/.env já existe no container — mantendo os secrets atuais."
  REUSE_ENV=1
else
  REUSE_ENV=0
  echo "==> Configuração de secrets (Enter em branco = gerar automaticamente)"
  read -r -s -p "Senha do Postgres [auto]: " PG_PASS; echo
  [ -n "$PG_PASS" ] || PG_PASS=$(openssl rand -hex 24)
  read -r -s -p "MinIO access key [auto]: " S3_ACCESS; echo
  [ -n "$S3_ACCESS" ] || S3_ACCESS="kura_$(openssl rand -hex 6)"
  read -r -s -p "MinIO secret key [auto]: " S3_SECRET; echo
  [ -n "$S3_SECRET" ] || S3_SECRET=$(openssl rand -hex 24)
  read -r -s -p "Chave privada do relay (64 hex) [auto-gerar]: " RELAY_KEY; echo
  [ -n "$RELAY_KEY" ] || RELAY_KEY=$(openssl rand -hex 32)
  read -r -p "Pubkey do OWNER (sua, 64 hex — obrigatório): " OWNER_PUBKEY
  [[ "$OWNER_PUBKEY" =~ ^[0-9a-f]{64}$ ]] || { echo "ERRO: pubkey deve ser 64 hex minúsculos"; exit 1; }
  GIT_HMAC=$(openssl rand -hex 32)
fi

echo "==> Escrevendo arquivos em /opt/kura…"
lxc exec "$CONTAINER" -- mkdir -p /opt/kura

if [ "$REUSE_ENV" = 0 ]; then
  lxc exec "$CONTAINER" -- bash -c 'cat > /opt/kura/.env && chmod 600 /opt/kura/.env' <<EOF
POSTGRES_PASSWORD=$PG_PASS
DATABASE_URL=postgres://kura:$PG_PASS@postgres:5432/kura
REDIS_URL=redis://redis:6379
KURA_BIND_ADDR=0.0.0.0:3000
KURA_AUTO_MIGRATE=true
RELAY_URL=wss://$DOMAIN
KURA_RELAY_PRIVATE_KEY=$RELAY_KEY
RELAY_OWNER_PUBKEY=$OWNER_PUBKEY
KURA_GIT_HOOK_HMAC_SECRET=$GIT_HMAC
KURA_PUSH_ENABLED=false
KURA_S3_ENDPOINT=http://minio:9000
KURA_S3_ACCESS_KEY=$S3_ACCESS
KURA_S3_SECRET_KEY=$S3_SECRET
KURA_S3_BUCKET=kura-media
KURA_S3_REGION=us-east-1
KURA_S3_ADDRESSING_STYLE=path
KURA_MEDIA_BASE_URL=https://$DOMAIN
RUST_LOG=kura_relay=info,kura_db=info,kura_auth=info
EOF
fi

lxc exec "$CONTAINER" -- bash -c 'cat > /opt/kura/docker-compose.yml' <<EOF
name: kura
services:
  relay:
    image: $IMAGE
    restart: unless-stopped
    env_file: .env
    ports:
      - "3000:3000"
    depends_on:
      postgres: { condition: service_healthy }
      redis: { condition: service_healthy }
      minio: { condition: service_healthy }

  postgres:
    image: postgres:17-alpine
    restart: unless-stopped
    environment:
      POSTGRES_USER: kura
      POSTGRES_DB: kura
      POSTGRES_PASSWORD: \${POSTGRES_PASSWORD}
    volumes: [ "pgdata:/var/lib/postgresql/data" ]
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U kura -d kura"]
      interval: 5s
      timeout: 3s
      retries: 20

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    volumes: [ "redisdata:/data" ]
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 20

  minio:
    image: minio/minio:latest
    restart: unless-stopped
    command: server /data
    environment:
      MINIO_ROOT_USER: \${KURA_S3_ACCESS_KEY}
      MINIO_ROOT_PASSWORD: \${KURA_S3_SECRET_KEY}
    volumes: [ "miniodata:/data" ]
    healthcheck:
      test: ["CMD", "mc", "ready", "local"]
      interval: 5s
      timeout: 3s
      retries: 20

  minio-init:
    image: minio/mc:latest
    depends_on:
      minio: { condition: service_healthy }
    entrypoint: >
      /bin/sh -c "mc alias set local http://minio:9000 \${KURA_S3_ACCESS_KEY} \${KURA_S3_SECRET_KEY} &&
      mc mb -p local/kura-media || true"
    env_file: .env
    restart: "no"

volumes:
  pgdata:
  redisdata:
  miniodata:
EOF

echo "==> Subindo o stack…"
lxc exec "$CONTAINER" -- bash -c 'cd /opt/kura && docker compose pull -q && docker compose up -d'

echo "==> Aguardando relay responder (NIP-11)…"
for _ in $(seq 1 30); do
  if lxc exec "$CONTAINER" -- curl -sf -H 'Accept: application/nostr+json' http://localhost:3000 >/dev/null 2>&1; then
    echo "    relay OK"
    break
  fi
  sleep 3
done
lxc exec "$CONTAINER" -- curl -s -H 'Accept: application/nostr+json' http://localhost:3000 | head -c 400 || true
echo
echo
echo "OK. Próximo passo: crie o A record de $DOMAIN no Hostinger e rode ./03-nginx-vhost.sh"
