#!/usr/bin/env bash
# Cria o vhost nginx no host para dev.kura.oute.pro (proxy WSS -> LXC:3000)
# e emite o certificado Let's Encrypt. Exige o A record já publicado.
set -euo pipefail

CONTAINER="${KURA_LXC_NAME:-kura-dev}"
DOMAIN="${KURA_DEV_DOMAIN:-dev.kura.oute.pro}"

IP=$(lxc exec "$CONTAINER" -- hostname -I | tr " " "\n" | grep -m1 "^10\." || true)
[ -n "$IP" ] || { echo "ERRO: LXC '$CONTAINER' sem IP"; exit 1; }

PUB_IP=$(curl -s https://api.ipify.org || true)
DNS_IP=$(dig +short "$DOMAIN" | tail -1 || true)
echo "==> IP público do servidor: ${PUB_IP:-?} | DNS de $DOMAIN: ${DNS_IP:-NÃO RESOLVE}"
if [ -z "$DNS_IP" ] || { [ -n "$PUB_IP" ] && [ "$DNS_IP" != "$PUB_IP" ]; }; then
  echo "ATENÇÃO: crie/ajuste no Hostinger o registro A: $DOMAIN -> ${PUB_IP:-<ip do servidor>}"
  read -r -p "DNS já propagou e quer continuar mesmo assim? [s/N] " go
  [[ "$go" =~ ^[sS]$ ]] || exit 1
fi

VHOST=/etc/nginx/sites-available/$DOMAIN
echo "==> Escrevendo $VHOST (proxy -> $IP:3000)…"
sudo tee "$VHOST" >/dev/null <<EOF
server {
    listen 80;
    server_name $DOMAIN;

    location / {
        proxy_pass http://$IP:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
        client_max_body_size 200m;
    }
}
EOF
sudo ln -sf "$VHOST" /etc/nginx/sites-enabled/$DOMAIN
sudo nginx -t
sudo systemctl reload nginx

echo "==> Emitindo certificado (certbot --nginx)…"
sudo certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --redirect \
  ${CERTBOT_EMAIL:+-m "$CERTBOT_EMAIL"} || {
    echo "Se falhou por e-mail ausente: exporte CERTBOT_EMAIL=seu@email e rode de novo."
    exit 1
  }

# Vhosts existentes fixam listen no IP do Tailscale; sem um listen igual aqui,
# conexões vindas do tailnet caem no primeiro vhost daquele socket.
TSIP=$(tailscale ip -4 2>/dev/null || true)
if [ -n "$TSIP" ] && ! sudo grep -q "listen $TSIP:443" "/etc/nginx/sites-enabled/$DOMAIN"; then
  echo "==> Adicionando listen $TSIP:443 (tailnet) ao vhost…"
  sudo sed -i "s|^    listen 443 ssl; # managed by Certbot|    listen 443 ssl; # managed by Certbot\n    listen $TSIP:443 ssl;|" "/etc/nginx/sites-enabled/$DOMAIN"
fi

sudo nginx -t && sudo systemctl reload nginx
echo
echo "OK. https://$DOMAIN no ar. Próximo passo: ./04-validate.sh"
