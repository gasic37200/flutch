# Egress는 방이 있어야 시작되고, `egress_ended` 웹훅에는 `room`이 없다
- task: T25 · PR: (이 PR) · 날짜: 2026-11-10
- 영역: backend/recording, backend/live(webhook)

## 증상
- 설계 문서는 "LIVE 전이 때 Egress 시작"이었다. 그런데 `POST /live` 시점에는 송출자가 아직 접속 전이라 LiveKit 방이 없다. `StartRoomCompositeEgress`는 존재하는 방이 필요하다(없으면 "requested room does not exist"로 실패, LiveKit 문서·소스 기준 — 로컬에 Egress가 없어 실측은 못 했다).
- 기존 웹훅 핸들러는 `room.name`이 `broadcast-<id>`가 아니면 무시한다. `egress_ended`는 `egressInfo`만 오고 최상위 `room`이 없어 그대로 두면 완료 통보가 버려진다.

## 원인
- 녹화 시작 시점과 방 생성 시점이 다르다. Egress 이벤트의 페이로드 모양이 room 이벤트와 다르다.

## 해결
- Egress 시작을 송출자 접속(`participant_joined`, 현재 송출자 신원) 웹훅으로 옮겼다. `recordings.broadcast_id` 유니크 + `INSERT … ON CONFLICT DO NOTHING`으로 접속이 반복돼도 한 번만 시작한다.
- `egress_ended`는 방 판별보다 앞서 `egressInfo.egressId`로 `recordings.egress_id`를 찾아 처리한다. int64(size·duration)는 protojson이 문자열로 주므로 문자열·숫자 모두 받는다.
- 송출자가 끊겼다 60초 안에 돌아와 방이 새로 만들어지면 Egress는 이어지지 않는다(앞 구간만 저장). 자동 재시도는 정책상 없다.

## 재발 방지
- 배포 환경(LiveKit Cloud Egress)에서 첫 방송으로 시작·완료 웹훅·파일 업로드를 실측한다(README 체크리스트 P5). `docs/api.md` §3-1·§8·§10에 반영.
