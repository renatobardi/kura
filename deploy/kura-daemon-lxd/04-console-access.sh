#!/usr/bin/env bash
# PLACEHOLDER — não faz nada de verdade ainda.
#
# O plano (Fase 5, item 4) previa `tailscale serve --https=443
# 127.0.0.1:<porta>` dentro do LXC para expor um console/API do kurad via
# tailnet. Só que, verificado em crates/kurad/src/main.rs (comentário do
# módulo, linhas iniciais): "Deliberately absent (later phases): the
# JSON-RPC/WebSocket API, the web console, any HTTP listener, and any
# service-manager integration." — ou seja, o `kurad` de hoje não abre
# nenhuma porta HTTP. Não há <porta> para apontar o `tailscale serve`.
#
# Esse listener HTTP é escopo explícito da Fase 3 do plano (API
# JSON-RPC/WebSocket) e da futura `kura-console` (D11), nenhuma das duas
# construída ainda. Quando existir, este script deve virar algo como:
#
#   tailscale serve --bg --https=443 127.0.0.1:<porta-do-kurad>
#
# rodado dentro do LXC (ou, alternativamente, um vhost nginx no host como o
# deploy/kura-dev/03-nginx-vhost.sh, só na interface tailscale). Até lá, este
# LXC é host de agentes (kurad roda em foreground/systemd, sem UI), não serve
# console nenhum.
set -euo pipefail

echo "04-console-access.sh: nada a fazer — kurad ainda não expõe HTTP/API."
echo "Ver comentário no topo deste arquivo e o plano da Fase 3."
exit 0
