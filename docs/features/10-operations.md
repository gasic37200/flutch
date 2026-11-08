# 10. 운영·공통

## 운영자
- 운영자 역할·화면 없음. 긴급 시 서버에서 직접 처리(LiveKit 대시보드 방 종료 등). 신고 기능 없음.

## 에러 화면
- 404(없는 페이지·삭제된 방송), 500, 네트워크 끊김: **마스코트 + 안내 + 홈으로**.
- 서버 오류 화면과 에러 응답에 `request_id` 표시(Grafana 추적).
- 점검 모드 없음.

## 로그·모니터링
- `tracing` JSON 로그, 요청마다 `request_id`·method·route·status·latency·user_id.
- 레벨: ERROR(5xx·외부 연동 실패) / WARN(웹훅 서명 실패·요청 제한·반복 권한 거절) / INFO(방송 시작·종료, 결제 확정, 밴, SUPER CLUTCH) / DEBUG(로컬만).
- 절대 기록 금지: 비밀번호, 세션 쿠키, LiveKit 토큰, 토스 시크릿, 채팅 본문.
- Prometheus(15일) + Loki(7일, 라벨은 service·level만) + Grafana(`/grafana/`, 로그인) + Alloy + node-exporter. `/metrics`·Prometheus·Loki는 외부 비공개.
- 대시보드: 서비스 개요(요청량·에러율·p95·LIVE 수·WS 연결 수·후원·자원), 로그 검색. 알림 없음.

## 개인정보·데이터 보관
- IP는 요청 제한 계산에만, **메모리에만**(DB 저장 안 함).
- 만료 세션 매일 정리. 로그 7일. DB 백업 매일 → Object Storage 7일.
- 탈퇴 회원: 닉네임 30일 후 해제, 원장·채팅은 "탈퇴한 사용자"로 보존.
- 개인정보처리방침에 위 내용 기재.

## 반응형·접근성
- 모든 화면 모바일 웹 동작(송출·시청·채팅·후원·지갑).
- 키보드로 주요 동작, 버튼·이미지 대체 텍스트, 색만으로 정보 구분 금지(LIVE 배지에 글자), `prefers-reduced-motion` 존중.

## 보안 공통
- CSRF: SameSite=Lax + 변경 요청 `Origin` 검사. 웹훅(LiveKit·토스)은 서명·시크릿 검증.
- 모든 입력은 서버에서 길이·형식 검증. 에러 응답 형식은 `docs/api.md`.
