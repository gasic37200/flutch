#!/usr/bin/env bash
# OCI Ampere A1 인스턴스를 "Out of capacity"가 풀릴 때까지 재시도해서 만든다.
# 사용: cp launch-a1.env.example launch-a1.env → 값 채우기 → nohup ./launch-a1.sh > launch.log 2>&1 &
set -uo pipefail
cd "$(dirname "$0")"

# shellcheck source=/dev/null
source ./launch-a1.env   # COMPARTMENT_ID SUBNET_ID IMAGE_ID AVAILABILITY_DOMAIN SSH_PUBLIC_KEY_FILE (커밋 금지)

DISPLAY_NAME="${DISPLAY_NAME:-flutch-server-prod-01}"
OCPUS="${OCPUS:-3}"                      # 무료 4 OCPU/24GB 중 포트폴리오 1/6을 뺀 나머지
MEMORY_GB="${MEMORY_GB:-18}"
BOOT_VOLUME_GB="${BOOT_VOLUME_GB:-100}"  # 무료 블록 스토리지 합계 200GB (포트폴리오 부트 볼륨 포함)
MAX_ATTEMPTS="${MAX_ATTEMPTS:-200}"
SLEEP_SECONDS="${SLEEP_SECONDS:-120}"    # 2분 간격 → 최대 약 6.5시간
CLOUD_INIT="./cloud-init.yaml"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

# 이미 같은 이름의 인스턴스가 있으면 만들지 않는다(타임아웃 뒤 재시도로 중복 생성되는 것 방지)
existing_instance() {
  oci compute instance list \
    --compartment-id "$COMPARTMENT_ID" \
    --display-name "$DISPLAY_NAME" \
    --query "data[?\"lifecycle-state\"!='TERMINATED' && \"lifecycle-state\"!='TERMINATING'].id | [0]" \
    --raw-output 2>/dev/null
}

for attempt in $(seq 1 "$MAX_ATTEMPTS"); do
  found="$(existing_instance)"
  if [ -n "$found" ] && [ "$found" != "null" ]; then
    log "이미 있음: $found (상태는 콘솔에서 확인)"
    exit 0
  fi

  log "시도 ${attempt}/${MAX_ATTEMPTS} (${OCPUS} OCPU / ${MEMORY_GB}GB)"
  OUTPUT=$(oci compute instance launch \
    --compartment-id "$COMPARTMENT_ID" \
    --availability-domain "$AVAILABILITY_DOMAIN" \
    --display-name "$DISPLAY_NAME" \
    --shape "VM.Standard.A1.Flex" \
    --shape-config "{\"ocpus\": ${OCPUS}, \"memoryInGBs\": ${MEMORY_GB}}" \
    --image-id "$IMAGE_ID" \
    --subnet-id "$SUBNET_ID" \
    --assign-public-ip true \
    --ssh-authorized-keys-file "$SSH_PUBLIC_KEY_FILE" \
    --user-data-file "$CLOUD_INIT" \
    --boot-volume-size-in-gbs "$BOOT_VOLUME_GB" \
    --wait-for-state RUNNING \
    --max-wait-seconds 900 \
    --query 'data.id' --raw-output 2>&1)
  status=$?

  if [ $status -eq 0 ]; then
    log "성공: $OUTPUT"
    IP=$(oci compute instance list-vnics --instance-id "$OUTPUT" --query 'data[0]."public-ip"' --raw-output 2>/dev/null)
    log "공인 IP: ${IP:-콘솔에서 확인} → 내도메인 flutch.kro.kr A 레코드에 등록"
    exit 0
  fi

  echo "$OUTPUT" | tail -n 5
  if echo "$OUTPUT" | grep -Eqi "Out of (host )?capacity|InternalError"; then
    log "재고 부족. ${SLEEP_SECONDS}초 뒤 재시도"
    sleep "$SLEEP_SECONDS"; continue
  fi
  if echo "$OUTPUT" | grep -Eqi "TooManyRequests|429"; then
    log "요청 과다. $((SLEEP_SECONDS * 2))초 뒤 재시도"
    sleep $((SLEEP_SECONDS * 2)); continue
  fi
  if echo "$OUTPUT" | grep -Eqi "timed out|max-wait"; then
    log "RUNNING 대기 시간 초과. 생성됐을 수 있으니 다음 시도에서 중복 여부부터 확인"
    sleep 30; continue
  fi

  log "재고 문제가 아닌 오류(한도 초과·설정 실수 등). 멈춤."
  exit 1
done

log "최대 시도 횟수 도달. 사양을 낮추거나(OCPUS/MEMORY_GB) 나중에 다시 실행."
exit 1
