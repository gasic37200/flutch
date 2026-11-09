#!/usr/bin/env bash
# 서버에서 한 번만: 최초 Let's Encrypt 인증서 발급. 이후 갱신은 certbot 컨테이너가 한다.
# 사용: cd ~/flutch && ./init-cert.sh   (80번 포트가 비어 있어야 한다 = web 컨테이너 기동 전)
set -euo pipefail
cd "$(dirname "$0")"
DOMAIN=flutch.kro.kr
C="docker compose -f compose.prod.yml"

$C --profile init up -d acme-http
trap '$C --profile init rm -sf acme-http' EXIT
sleep 2
$C run --rm --entrypoint certbot certbot certonly --webroot -w /var/www/certbot \
  -d "$DOMAIN" --non-interactive --agree-tos --register-unsafely-without-email
