# Kura — ambiente dev no oute-server (LXD/LXC)

Sobe o `kura-dev` num container LXC com Docker Compose, exposto em
`https://dev.kura.oute.pro` via nginx no host. A imagem vem do CI:
`ghcr.io/renatobardi/kura:main` (todo push na main publica). O prd futuro
usará o mesmo caminho com tags `relay-v*` — essa é a base da pipeline dev→prd.

## Pré-requisitos
- LXD funcionando no oute-server (Ampere/arm64) e nginx + certbot no host.
- Registro DNS **A** em `dev.kura.oute.pro` → IP público do oute-server
  (criar no painel do Hostinger ANTES do passo 3).

## Ordem de execução (no oute-server, como usuário com lxc/sudo)

```bash
./01-lxc-create.sh      # cria o LXC kura-dev (Ubuntu 24.04) e instala Docker
./02-kura-up.sh         # pergunta secrets, escreve .env + compose no LXC e sobe o stack
./03-nginx-vhost.sh     # vhost nginx + certificado Let's Encrypt (exige DNS pronto)
./04-validate.sh        # NIP-11 via HTTPS, upgrade WebSocket, saúde dos containers
```

Os secrets são pedidos no terminal na hora (nada fica nos scripts):
senha do Postgres, chaves do MinIO, chave privada do relay (gera uma se
deixar em branco), pubkey do owner (sua, hex) e HMAC dos hooks git.
O `.env` final fica em `/opt/kura/.env` dentro do LXC (chmod 600).

## Operação
```bash
lxc exec kura-dev -- docker compose -f /opt/kura/docker-compose.yml ps
lxc exec kura-dev -- docker compose -f /opt/kura/docker-compose.yml logs -f relay
# atualizar para a main mais recente:
lxc exec kura-dev -- bash -c 'cd /opt/kura && docker compose pull relay && docker compose up -d relay'
```

## Desktop apontando para o dev
No app: adicionar comunidade com URL `wss://dev.kura.oute.pro`.
