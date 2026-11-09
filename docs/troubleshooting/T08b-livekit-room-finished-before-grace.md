# LiveKit `room_finished`는 마지막 참가자 퇴장 약 20초 뒤에 온다 — 받자마자 종료하면 60초 유예가 깨진다
- task: T08b · PR: (이 PR) · 날짜: 2026-11-09
- 영역: backend/live

## 증상
api.md §10 초안은 "`room_finished`: LIVE면 종료"였다. 로컬 LiveKit(v1.13)으로 송출자·시청자가 모두 나가게 하자 `participant_left` 후 약 18초 만에 `room_finished`가 왔다(로그: 퇴장 20:56:42 → room_finished 20:56:59). 시청자가 없는 방송에서 스트리머가 새로고침·네트워크 끊김으로 20초 넘게 빠지면 60초 유예 전에 방송이 끝난다.

## 원인
LiveKit은 마지막 참가자가 나가면 `departure_timeout`(기본 20초) 뒤 빈 방을 닫는다. 방은 다음 참가 때 자동으로 다시 만들어지므로 "방이 닫힘 ≠ 송출 불가".

## 해결
`room_finished`는 LIVE면 **유예 시작**(`disconnected_at`이 비어 있을 때만)으로 처리하고 종료는 5초 주기 작업에 맡긴다(`live/webhook.rs::room_finished`). 우리가 `end_live`로 방을 지운 뒤 오는 `room_finished`는 이미 ENDED라 아무 일도 없다.

## 재발 방지
- api.md §10 웹훅 행을 "유예 시작"으로 고치고 이유를 적었다.
- 시나리오 `live_viewer_count` 끝에서 `room_finished` → LIVE 유지 + `reconnecting=true` 확인.
