#!/usr/bin/env bash
# Valida o kura-dev de fora: HTTPS/NIP-11, upgrade WebSocket e containers.
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-dev}"
DOMAIN="${KURA_DEV_DOMAIN:-dev.kura.oute.pro}"
FAIL=0

echo "== 1/4 Containers =="
lxc exec "$CONTAINER" -- docker compose -f /opt/kura/docker-compose.yml ps

echo
echo "== 2/4 NIP-11 via HTTPS =="
if curl -sf -H 'Accept: application/nostr+json' "https://$DOMAIN" | python3 -m json.tool | head -15; then
  echo "-> OK"
else
  echo "-> FALHOU"; FAIL=1
fi

echo
echo "== 3/4 Upgrade WebSocket =="
CODE=$(curl -s -o /dev/null -w '%{http_code}' \
  -H 'Connection: Upgrade' -H 'Upgrade: websocket' \
  -H 'Sec-WebSocket-Version: 13' -H 'Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==' \
  "https://$DOMAIN")
if [ "$CODE" = "101" ]; then echo "-> OK (101 Switching Protocols)"; else echo "-> FALHOU (HTTP $CODE)"; FAIL=1; fi

echo
echo "== 4/4 Web servida pelo relay =="
if curl -sf "https://$DOMAIN" | grep -qi "<html"; then echo "-> OK"; else echo "-> aviso: raiz não retornou HTML (pode ser esperado)"; fi

echo
if [ "$FAIL" = 0 ]; then
  echo "TUDO OK — kura-dev no ar em https://$DOMAIN (wss://$DOMAIN no app)."
else
  echo "Há falhas acima — logs: lxc exec $CONTAINER -- docker compose -f /opt/kura/docker-compose.yml logs relay | tail -50"
  exit 1
fi
