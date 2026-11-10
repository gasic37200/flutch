# 작업 기록

> 메인 세션만 기록한다(AGENTS §9-7). 토큰·도구 호출·소요 시간은 Agent 도구 결과에 표시된 값을 옮겨 적는다. **시각은 KST `YYYY-MM-DD HH:MM`**.

| 날짜 | task | 결과 | 시작 → 종료 | 소요 시간 | 토큰 | 도구 호출 | PR | 메모 |
|---|---|---|---|---|---|---|---|---|
| 2026-11-09 | T01 | 완료 | 2026-11-09 17:39 → 18:15 | 35분 | 207,062 | 85 | [#25](https://github.com/gasic37200/flutch/pull/25) | Opus·high. CI 첫 실행 5분 35초, 캐시 41초 |
| 2026-11-09 | T05 | 완료 | 2026-11-09 18:16 → 18:34 | 18분 | 175,811 | 61 | [#28](https://github.com/gasic37200/flutch/pull/28) | Sonnet·medium |
| 2026-11-09 | T02 | 완료 | 2026-11-09 18:16 → 18:34 | 18분 | 171,022 | 34 | [#27](https://github.com/gasic37200/flutch/pull/27) | Opus·high. 메인이 build.rs(마이그레이션 재컴파일) 추가 후 머지 |
| 2026-11-09 | T07 | 부분 | 2026-11-09 18:16 → 18:47 | 30분 | 146,750 | 45 | [#29](https://github.com/gasic37200/flutch/pull/29) | Sonnet·medium. arm64 빌드·GHCR 푸시 확인. 인증서(kro.kr LE 한도)·Secrets 없어 실배포 미확인. 메인이 main 충돌 merge 해결 |
| 2026-11-09 | T02b | 완료 | 2026-11-09 18:40 → 18:52 | 12분 | 197,638 | 15 | [#31](https://github.com/gasic37200/flutch/pull/31) | Opus·medium. 에러 코드 51개, WS 이벤트 15종 |
| 2026-11-09 | T03 | 완료 | 2026-11-09 18:54 → 19:26 | 32분 | 251,788 | 86 | [#33](https://github.com/gasic37200/flutch/pull/33) | Opus·high. ponytail-review: 비밀번호 변경 시도 제한 1건 → 메인이 수정 후 머지 |
| 2026-11-09 | T06a | 완료 | 2026-11-09 19:32 → 19:46 | 14분 | 151,351 | 72 | [#35](https://github.com/gasic37200/flutch/pull/35) | Sonnet·medium. 브라우저로 가입·로그인·테스트 로그인 흐름 확인 |
| 2026-11-09 | T04 | 완료 | 2026-11-09 19:32 → 19:50 | 18분 | 195,874 | 53 | [#36](https://github.com/gasic37200/flutch/pull/36) | Sonnet·medium. /tmp 공유로 T06a worktree 오염 → 스스로 되돌림, AGENTS §9-4 규칙 추가 |
| 2026-11-09 | T06b | 완료 | 2026-11-09 19:52 → 20:13 | 21분 | 186,006 | 94 | [#39](https://github.com/gasic37200/flutch/pull/39) | Sonnet·medium |
| 2026-11-09 | T08a | 완료 | 2026-11-09 19:52 → 20:13 | 21분 | 195,633 | 55 | [#38](https://github.com/gasic37200/flutch/pull/38) | Opus·high. 로컬 LiveKit으로 토큰 권한·takeover·방 삭제 실측 |
| 2026-11-09 | T08c | 완료 | 2026-11-09 20:15 → 20:33 | 18분 | 181,409 | 46 | [#41](https://github.com/gasic37200/flutch/pull/41) | Opus·high. tokio-tungstenite로 실제 WS 시나리오 5개 |
| 2026-11-09 | T08d | 완료 | 2026-11-09 20:34 → 20:47 | 13분 | 129,827 | 46 | [#43](https://github.com/gasic37200/flutch/pull/43) | Sonnet·medium. 브라우저로 READY 대기→LIVE 구독→종료 확인, 미디어 트랙은 미확인(U1) |
| 2026-11-09 | T11 | 완료 | 2026-11-09 20:49 → 21:03 | 14분 | 145,443 | 48 | [#45](https://github.com/gasic37200/flutch/pull/45) | Sonnet·medium. 실영상·소리는 U1 |
| 2026-11-09 | T09 | 부분(U1) | 2026-11-09 20:49 → 21:09 | 20분 | 181,732 | 68 | [#46](https://github.com/gasic37200/flutch/pull/46) | Sonnet·medium. 합성 스트림으로 3트랙 구독·볼륨 반영 확인, 실장치는 U1 |
| 2026-11-09 | T08b | 완료 | 2026-11-09 20:34 → 21:10 | 36분 | 276,948 | 100 | [#47](https://github.com/gasic37200/flutch/pull/47) | Opus·high. 로컬 LiveKit 실제 웹훅(참가·퇴장·방 종료) 서명 검증·유예 만료 종료 실측 |
| 2026-11-09 | **합계** | 15개(T01·T02·T02b·T03·T04·T05·T06a·T06b·T07 부분·T08a~d·T09·T11) | 17:39 → 21:11 | 서브에이전트 합 5시간 20분 | 2,794,294 | — | — | 메인 세션 사용량 미기록 |
| 2026-11-10 | T10 | 완료 | 2026-11-09 21:10 → 2026-11-10 09:31 | 표시 12시간 22분 | 155,718 | 124 | [#49](https://github.com/gasic37200/flutch/pull/49) | Sonnet·medium. **사용자 부재(밤) 동안 멈춘 시간 포함** — 실제 작업 시간은 짧음 |
| 2026-11-10 | T14 | 완료 | 2026-11-10 09:35 → 09:42 | 7분 | 122,339 | 16 | [#52](https://github.com/gasic37200/flutch/pull/52) | Sonnet·medium. 테스트 계정 비밀번호는 제출 전 메인이 기재 |
| 2026-11-10 | T05d | 완료 | 2026-11-10 09:35 → 09:56 | 21분 | 219,111 | 60 | [#53](https://github.com/gasic37200/flutch/pull/53) | Sonnet·medium. 시안 보드 4장 대조 |
