#!/usr/bin/env bash
# 서버에서 한 번만: 최초 인증서 발급. 이후 갱신은 certbot 컨테이너가 한다(발급한 CA를 renewal 설정에 기억한다).
# 사용: cd ~/flutch && ./init-cert.sh   (80번 포트가 비어 있어야 한다 = web 컨테이너 기동 전)
# CA: .env에 ZEROSSL_EAB_KID·ZEROSSL_EAB_HMAC_KEY가 있으면 ZeroSSL, 없으면 Let's Encrypt.
#     kro.kr은 Let's Encrypt 주간 발급 한도를 내도메인 사용자 전체가 나눠 써서 거절이 잦다(troubleshooting T07).
set -euo pipefail
cd "$(dirname "$0")"
DOMAIN=flutch.kro.kr
C="docker compose -f compose.prod.yml"

env_value() { grep -E "^$1=" .env 2>/dev/null | head -1 | cut -d= -f2- || true; }
KID=$(env_value ZEROSSL_EAB_KID)
HMAC=$(env_value ZEROSSL_EAB_HMAC_KEY)
CA_ARGS=()
if [ -n "$KID" ] && [ -n "$HMAC" ]; then
  echo "CA: ZeroSSL"
  CA_ARGS=(--server https://acme.zerossl.com/v2/DV90 --eab-kid "$KID" --eab-hmac-key "$HMAC")
else
  echo "CA: Let's Encrypt"
fi

$C --profile init up -d acme-http
trap '$C --profile init rm -sf acme-http' EXIT
sleep 2
$C run --rm --entrypoint certbot certbot certonly --webroot -w /var/www/certbot \
  -d "$DOMAIN" --non-interactive --agree-tos --register-unsafely-without-email "${CA_ARGS[@]}"
