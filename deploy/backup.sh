#!/usr/bin/env bash
# 매일 04:00 KST cron: pg_dump → ~/flutch/backups/ (+ Object Storage). 파일명에 요일을 써서 7개가 돌아가며 덮어써진다 = 7일 보관.
# cron: 0 4 * * * /home/ubuntu/flutch/backup.sh >> /home/ubuntu/flutch/backups/backup.log 2>&1
# Oracle S3 호환 API는 aws-cli 최신판의 기본 체크섬 헤더를 거절한다는 보고가 있어 when_required로 끄고 이미지 태그를 고정한다.
# Object Storage 업로드는 .env에 BACKUP_S3_* 가 있을 때만(S3 호환 키 사용).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p backups
FILE="backups/flutch-$(date +%a).sql.gz"

docker compose -f compose.prod.yml exec -T postgres pg_dump -U flutch flutch | gzip > "$FILE.tmp"
mv "$FILE.tmp" "$FILE"
echo "$(date '+%F %T') dumped $FILE ($(du -h "$FILE" | cut -f1))"

set -a; . ./.env; set +a
if [ -n "${BACKUP_S3_BUCKET:-}" ]; then
  docker run --rm -v "$PWD/backups:/backups:ro" \
    -e AWS_ACCESS_KEY_ID="$BACKUP_S3_ACCESS_KEY" -e AWS_SECRET_ACCESS_KEY="$BACKUP_S3_SECRET_KEY" \
    -e AWS_DEFAULT_REGION="${BACKUP_S3_REGION:-ap-chuncheon-1}" \
    -e AWS_REQUEST_CHECKSUM_CALCULATION=when_required -e AWS_RESPONSE_CHECKSUM_VALIDATION=when_required \
    amazon/aws-cli:2.22.35 --endpoint-url "$BACKUP_S3_ENDPOINT" s3 cp "/$FILE" "s3://$BACKUP_S3_BUCKET/$(basename "$FILE")" \
    || { echo "$(date '+%F %T') UPLOAD FAILED $(basename "$FILE") (로컬 백업은 남음)" >&2; exit 1; }
  echo "$(date '+%F %T') uploaded $(basename "$FILE")"
else
  echo "BACKUP_S3_BUCKET 미설정: Object Storage 업로드 건너뜀(로컬 백업만)"
fi
