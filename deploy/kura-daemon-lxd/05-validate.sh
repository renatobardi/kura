#!/usr/bin/env bash
# Valida o kura-daemon: só o que existe de verdade hoje (LXC, toolchain,
# binário, `kurad status`, unit systemd). NÃO checa healthz/API HTTP nem
# presença no relay — isso depende da Fase 3 (API) e ainda não existe (ver
# 04-console-access.sh).
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-daemon}"
DATA_DIR="${KURA_DAEMON_DATA_DIR:-/var/lib/kura}"
FAIL=0

echo "== 1/5 LXC rodando =="
if lxc info "$CONTAINER" 2>/dev/null | grep -q "Status: RUNNING"; then
  echo "-> OK ($CONTAINER)"
else
  echo "-> FALHOU (LXC '$CONTAINER' não existe ou não está rodando)"; FAIL=1
fi

echo
echo "== 2/5 Docker (nested) =="
if lxc exec "$CONTAINER" -- docker --version >/dev/null 2>&1; then
  echo "-> OK"
else
  echo "-> FALHOU"; FAIL=1
fi

echo
echo "== 3/5 Toolchain (apps.toml) =="
if lxc exec "$CONTAINER" -- test -f /opt/kura-daemon/apps.toml; then
  while IFS= read -r line; do
    case "$line" in
      name\ =\ *) APP_NAME=$(echo "$line" | sed -E 's/name = "(.*)"/\1/') ;;
      check_command\ =\ *)
        CMD=$(echo "$line" | sed -E 's/check_command = "(.*)"/\1/')
        if lxc exec "$CONTAINER" -- bash -lc "$CMD" >/dev/null 2>&1; then
          echo "  OK       $APP_NAME"
        else
          echo "  MISSING  $APP_NAME"; FAIL=1
        fi
        ;;
    esac
  done < <(lxc exec "$CONTAINER" -- cat /opt/kura-daemon/apps.toml)
else
  echo "-> FALHOU (apps.toml ausente — rode 02-toolchain.sh)"; FAIL=1
fi

echo
echo "== 4/5 kurad binário + status =="
if lxc exec "$CONTAINER" -- test -x /usr/local/bin/kurad; then
  echo "-> binário presente"
  STATUS_JSON=$(lxc exec "$CONTAINER" -- sudo -u kura kurad status --data-dir "$DATA_DIR" 2>&1) || {
    echo "-> FALHOU (kurad status saiu com erro)"
    echo "$STATUS_JSON"
    FAIL=1
  }
  if [ -n "${STATUS_JSON:-}" ] && echo "$STATUS_JSON" | python3 -m json.tool >/dev/null 2>&1; then
    echo "-> JSON válido:"
    echo "$STATUS_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); print("   dataDir:", d.get("dataDir")); print("   instanceId:", d.get("instanceId")); print("   agents:", len(d.get("agents", [])))'
  elif [ "$FAIL" = 0 ]; then
    echo "-> FALHOU (saída de 'kurad status' não é JSON válido)"; FAIL=1
  fi
else
  echo "-> FALHOU (binário ausente — rode 03-kurad.sh)"; FAIL=1
fi

echo
echo "== 5/5 Unit systemd =="
if lxc exec "$CONTAINER" -- test -f /etc/systemd/system/kurad.service; then
  echo "-> unit presente"
  if lxc exec "$CONTAINER" -- systemctl is-enabled kurad >/dev/null 2>&1; then
    lxc exec "$CONTAINER" -- systemctl is-active kurad || true
    echo "-> habilitada"
  else
    echo "-> aviso: unit ainda não habilitada (esperado até a identidade ser configurada — ver 03-kurad.sh)"
  fi
else
  echo "-> FALHOU (unit ausente — rode 03-kurad.sh)"; FAIL=1
fi

echo
if [ "$FAIL" = 0 ]; then
  echo "TUDO OK — checagens disponíveis hoje passaram. Sem checagem de HTTP/API (Fase 3, ainda não existe)."
else
  echo "Há falhas acima."
  exit 1
fi
