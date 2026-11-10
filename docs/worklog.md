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
| 2026-11-10 | T12 | 완료 | 2026-11-10 09:35 → 10:00 | 25분 | 162,266 | 57 | [#54](https://github.com/gasic37200/flutch/pull/54) | Sonnet·medium. 합성 스트림으로 캡처→업로드→홈 카드 표시 확인, 백그라운드 탭은 U1 |
| 2026-11-10 | T11b | 부분(U1) | 2026-11-10 09:57 → 10:16 | 19분 | 178,458 | 50 | [#57](https://github.com/gasic37200/flutch/pull/57) | Sonnet·medium. 재생 중·스트리머 재연결 화면은 U1 |
| 2026-11-10 | T06c | 완료 | 2026-11-10 09:57 → 10:23 | 26분 | 230,804 | 101 | [#58](https://github.com/gasic37200/flutch/pull/58) | Sonnet·medium. 약관·방송 수정 화면은 눈으로 미확인 |
| 2026-11-10 | T09c | 완료 | 2026-11-10 10:02 → 10:35 | 33분 | 211,172 | 129 | [#60](https://github.com/gasic37200/flutch/pull/60) | Sonnet·medium. 이어하기·기기 전환 모달 화면은 미확인 |
| 2026-11-10 | T13 | 완료 | 2026-11-10 10:17 → 10:52 | 35분 | 165,314 | 63 | [#62](https://github.com/gasic37200/flutch/pull/62) | Sonnet·medium. 보고서가 짚은 nginx `/api/` XFF 미상속 버그(T07)를 메인이 같은 PR에서 수정, 서버 .env에 GRAFANA_ADMIN_PASSWORD 추가 |
| 2026-11-10 | T15 감사(백엔드) | 완료 | 2026-11-10 10:57 → 11:05 | 8분 | 195,893 | 18 | — | /ponytail-audit 읽기 위임, 9건(must 1) |
| 2026-11-10 | T15 감사(운영) | 완료 | 2026-11-10 10:57 → 11:06 | 9분 | 160,350 | 17 | — | /ponytail-audit 읽기 위임, 7건 |
| 2026-11-10 | T15 감사(프론트) | 완료 | 2026-11-10 10:57 → 11:05 | 8분 | 169,277 | 14 | — | /ponytail-audit 읽기 위임, 7건(must 1, 백엔드 2번과 같은 버그) |
| 2026-11-10 | T15c | 완료 | 2026-11-10 11:09 → 11:30 | 21분 | 145,617 | 52 | [#65](https://github.com/gasic37200/flutch/pull/65) | Sonnet·medium. ponytail-review: backup.sh 실행 권한 1건 → 메인 수정 후 머지 |
| 2026-11-10 | T15b | 완료 | 2026-11-10 11:09 → 11:33 | 24분 | 142,531 | 65 | [#66](https://github.com/gasic37200/flutch/pull/66) | Sonnet·high. 모바일 카메라 전환은 U1 |
| 2026-11-10 | T15a | 완료 | 2026-11-10 11:09 → 11:52 | 44분 | 234,049 | 109 | [#68](https://github.com/gasic37200/flutch/pull/68) | Opus·high. must 2건 시나리오로 확인, hidden 시청자 웹훅 별도 LiveKit 컨테이너로 실측 |
| 2026-11-10 | T20 | 완료 | 2026-11-10 11:59 → 12:22 | 24분 | 173,136 | 34 | [#72](https://github.com/gasic37200/flutch/pull/72) | Opus·high. ponytail-review: 지적 없음, 동시 차감 실제 2커넥션 시나리오 |
| 2026-11-10 | T16 | 완료 | 2026-11-10 11:59 → 12:23 | 24분 | 178,094 | 37 | [#73](https://github.com/gasic37200/flutch/pull/73) | Sonnet·medium |
| 2026-11-10 | T25 | 완료 | 2026-11-10 11:59 → 12:40 | 41분 | 219,293 | 61 | [#74](https://github.com/gasic37200/flutch/pull/74) | Sonnet·medium. 실제 Egress·S3 업로드는 배포 후 확인 |
| 2026-11-10 | T26 | 부분 | 2026-11-10 12:42 → 13:09 | 27분 | 238,051 | 92 | [#77](https://github.com/gasic37200/flutch/pull/77) | Sonnet·medium. `GET /users/{id}` 백엔드 없음 → T26b로 후속 |
| 2026-11-10 | T17 | 완료 | 2026-11-10 12:26 → 13:10 | 44분 | 257,063 | 170 | [#78](https://github.com/gasic37200/flutch/pull/78) | Sonnet·medium. 두 사용자 실시간 채팅·차단 입력창 확인 |
| 2026-11-10 | T21 | 완료 | 2026-11-10 12:26 → 13:11 | 45분 | 297,073 | 80 | [#79](https://github.com/gasic37200/flutch/pull/79) | Opus·high. ponytail-review: 지적 없음, 실제 토스 호출은 키 받은 뒤 |
| 2026-11-10 | T26b | 완료 | 2026-11-10 13:10 → 13:22 | 12분 | 116,116 | 19 | [#82](https://github.com/gasic37200/flutch/pull/82) | Sonnet·medium. follow 도메인에 GET /users/{id} |
| 2026-11-10 | T18 | 완료 | 2026-11-10 13:17 → 13:36 | 19분 | 156,036 | 43 | [#84](https://github.com/gasic37200/flutch/pull/84) | Sonnet·medium |
| 2026-11-10 | T19 | 완료 | 2026-11-10 13:23 → 14:09 | 46분 | 241,214 | 92 | [#86](https://github.com/gasic37200/flutch/pull/86) | Sonnet·medium. 동시 발동 advisory lock, 게이지 묶음 static은 T24에서 정리 |
| 2026-11-10 | T18u | 완료 | 2026-11-10 13:37 → 14:12 | 35분 | 185,131 | 120 | [#87](https://github.com/gasic37200/flutch/pull/87) | Sonnet·medium. 두 사용자로 차단·해제·영구 밴 확인 |
| 2026-11-10 | T22 | 완료 | 2026-11-10 13:17 → 14:47 | 90분 | 266,866 | 85 | [#89](https://github.com/gasic37200/flutch/pull/89) | Opus·high. ponytail-review: 지적 없음. **main squash 제목이 `wip(donation)`으로 들어감**(되돌리지 않음, AGENTS §8 보강) |
| 2026-11-10 | T09b | 완료 | 2026-11-10 14:14 → 14:56 | 42분 | 199,774 | 129 | [#91](https://github.com/gasic37200/flutch/pull/91) | Sonnet·medium. livekit-client TrackUnsubscribed 순서 버그 수정, 실장치는 U1 |
| 2026-11-10 | T20b | 완료 | 2026-11-10 14:14 → 15:16 | 65분 | 250,099 | 133 | [#93](https://github.com/gasic37200/flutch/pull/93) | Opus·high. ponytail-review: 탈퇴 후 웹훅 충전 1건(nice) → T21b |
| 2026-11-10 | T27 | 완료 | 2026-11-10 15:19 → 16:08 | 49분 | 169,298 | 75 | [#96](https://github.com/gasic37200/flutch/pull/96) | Sonnet·medium. 확장 단계 첫 task |
| 2026-11-10 | T23 | 부분(토스 키) | 2026-11-10 14:52 → 16:17 | 86분 | 354,321 | 236 | [#95](https://github.com/gasic37200/flutch/pull/95) | Sonnet·medium. 실제 결제창·결제는 토스 키 받은 뒤, 공용 Postgres 메모리 부족 재시작 1회 |
| 2026-11-10 | T21b | 완료 | 2026-11-10 16:10 → 16:30 | 20분 | 96,341 | 9 | [#100](https://github.com/gasic37200/flutch/pull/100) | Sonnet·medium. 메인 리뷰: 지적 없음 |
| 2026-11-10 | T24 | 완료 | 2026-11-10 14:58 → 16:32 | 94분 | 273,513 | 144 | [#99](https://github.com/gasic37200/flutch/pull/99) | Opus·high. ponytail-review: 지적 없음, T19 전역 static → AppState |
| 2026-11-10 | T28 | 완료 | 2026-11-10 16:32 → 17:05 | 33분 | 231,129 | 55 | [#103](https://github.com/gasic37200/flutch/pull/103) | Sonnet·medium. ponytail-review: 지적 없음, 실제 카카오 로그인은 배포 후 |
