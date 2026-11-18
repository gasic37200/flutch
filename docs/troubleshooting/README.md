# 트러블슈팅 기록

> 서브에이전트가 task를 마칠 때, 작업 중 막혔던 것·예상과 달랐던 것·시행착오가 있었으면 기록한다. 같은 영역을 작업하기 전에 이 목록을 먼저 본다.

## 규칙
- 파일 하나 = 문제 하나. 이름: `T<번호>-<짧은-영문-설명>.md` (예: `T08-livekit-webhook-signature.md`)
- 아래 목록 표에 한 줄 추가(같은 PR에서).
- 해결 과정에서 규칙이 생기면 "재발 방지"에 적고, 문서(AGENTS·features·architecture) 반영이 필요하면 같은 PR에 포함.
- 비밀값·토큰·개인정보는 적지 않는다.

## 양식
```markdown
# <문제 한 줄 요약>
- task: Txx · PR: #nn · 날짜: YYYY-MM-DD
- 영역: <backend/live, frontend/studio, deploy ...>

## 증상
<무엇이 어떻게 안 됐나. 에러 메시지 핵심 줄>

## 원인
<왜 그랬나>

## 해결
<무엇을 바꿨나. 핵심 코드·설정 한두 줄>

## 재발 방지
<규칙·체크 항목, 반영한 문서>
```

## 목록
| 파일 | task | 요약 |
|---|---|---|
| [T01-livekit-webhook-wsl.md](T01-livekit-webhook-wsl.md) | T01 | 로컬 LiveKit 웹훅은 WSL 백엔드에 닿음(확인), compose는 루트 .env를 안 읽어 `--env-file .env` 필요 |
| [T01-reqwest-ring-backend.md](T01-reqwest-ring-backend.md) | T01 | reqwest 0.13은 rustls 기본이 aws-lc → ring 쓰려고 0.12 고정 |
| [T07-letsencrypt-kro-kr-rate-limit.md](T07-letsencrypt-kro-kr-rate-limit.md) | T07 | kro.kr 전체가 Let's Encrypt 주간 발급 한도(50개)에 걸려 인증서 발급 거절 → 한도 해제 시각 이후 재시도 |
| [T02-sqlx-sqlsafestr.md](T02-sqlx-sqlsafestr.md) | T02 | sqlx 0.9 `query()`는 `&'static str`만 받음 → `format!` SQL 대신 `.bind()` |
| [T08a-livekit-grant-defaults.md](T08a-livekit-grant-defaults.md) | T08a | LiveKit 토큰 `canPublish`·`canPublishData`는 생략 시 true → 시청 토큰에 false 명시 |
| [T08b-livekit-room-finished-before-grace.md](T08b-livekit-room-finished-before-grace.md) | T08b | LiveKit은 빈 방을 약 20초 뒤 닫음 → `room_finished`에서 바로 종료하지 않고 60초 유예 시작 |
| [T08b-shared-livekit-room-collision.md](T08b-shared-livekit-room-collision.md) | T08b | 공용 로컬 LiveKit에서 worktree끼리 방 이름·웹훅이 섞임 → 웹훅 확인은 다른 LiveKit task와 시간을 나눠서 |
| [T05d-tailwind-class-order-bg-override.md](T05d-tailwind-class-order-bg-override.md) | T05d | Tailwind는 CSS 순서가 이겨서 기본 배경 + 덮어쓰기 클래스가 안 먹음 → 덮어쓸 속성은 조건부로 뺀다 |
| [T13-loki-allvalue-empty-matcher.md](T13-loki-allvalue-empty-matcher.md) | T13 | Grafana 변수 All(`.*`)이 Loki의 빈 매처 거절로 오류 → service 변수 allValue를 `.+`로 |
| [T09c-localhost-cookie-shared-across-ports.md](T09c-localhost-cookie-shared-across-ports.md) | T09c | localhost 쿠키는 포트를 안 가려 다른 worktree 로그인이 내 세션을 덮어씀 → 401이면 다시 로그인 |
| [T15a-livekit-webhook-probe.md](T15a-livekit-webhook-probe.md) | T15a | 공용 LiveKit 웹훅이 WSL에 안 닿아 별도 LiveKit 컨테이너(WSL IP 웹훅)로 실측: hidden도 웹훅 옴, 서버 API는 snake_case |
| [T25-egress-needs-room-and-no-room-in-webhook.md](T25-egress-needs-room-and-no-room-in-webhook.md) | T25 | Egress는 방이 있어야 시작(→ 송출자 접속 웹훅에서 시작), `egress_ended` 웹훅엔 `room`이 없음 |
| [T21-concurrent-test-overlap.md](T21-concurrent-test-overlap.md) | T21 | 동시 confirm 시나리오가 단독 실행에선 겹치지 않아 실패 → FakeToss 승인 지연으로 겹침 보장 |
| [T22-pg-stat-activity-snapshot-in-tx.md](T22-pg-stat-activity-snapshot-in-tx.md) | T22 | 트랜잭션 안 pg_stat_activity 폴링은 스냅숏이 고정돼 겹침 확인이 멈춤 → pg_stat_clear_snapshot, 잠금은 풀 밖 커넥션 |
| [T09b-trackunsubscribed-stale-track.md](T09b-trackunsubscribed-stale-track.md) | T09b | livekit-client는 TrackUnsubscribed를 pub.track 비우기 전에 발행 → sync에서 떠나는 트랙을 직접 제외 |
| [T20b-withdraw-redirects-to-login.md](T20b-withdraw-redirects-to-login.md) | T20b | 탈퇴 성공 후 `me`를 먼저 비우면 `/me`의 로그인 확인이 로그인 화면으로 보냄 → 홈을 새로 불러옴 |
| [T20b-sqlx-prepare-drops-test-queries.md](T20b-sqlx-prepare-drops-test-queries.md) | T20b | `cargo sqlx prepare`는 테스트 타깃 쿼리 캐시를 지움 → `-- --all-targets`로 모은다 |
| [T24-throttle-test-timing.md](T24-throttle-test-timing.md) | T24 | 묶음(throttle) 시나리오가 전체 테스트에서만 실패 → 시간 기준점을 이벤트 수신 뒤가 아니라 요청 전에, 묶일 입력은 직접 publish로 몰아 넣기 |
| [T29-migration-order-after-parallel-task.md](T29-migration-order-after-parallel-task.md) | T29 | 병렬 task가 먼저 머지한 마이그레이션보다 버전이 앞섬 → PR 전 rebase 뒤 버전 확인, 머지 전이면 이름을 더 큰 버전으로 |
| [T34-clippy-online-empty-task-db.md](T34-clippy-online-empty-task-db.md) | T34 | 막 만든 task DB에 `DATABASE_URL`을 두면 `query!` 매크로가 빈 DB를 검사해 clippy 실패 → `SQLX_OFFLINE=true` 또는 `sqlx migrate run` 뒤 |
| [T35-toss-window-under-dialog-top-layer.md](T35-toss-window-under-dialog-top-layer.md) | T35 | 토스 결제창(body 오버레이)이 `showModal()` 모달(top layer·inert) 뒤에 깔림 → 결제창이 떠 있는 동안 dialog를 `show()`로 내림 |
| [T45-withdraw-lock-fk-key-share.md](T45-withdraw-lock-fk-key-share.md) | T45 | 탈퇴에 사용자 FOR UPDATE를 넣으면 후원의 FK 확인(KEY SHARE)과 교착 → FOR NO KEY UPDATE + 확정 FOR SHARE, 경쟁 재현은 미커밋 지갑 INSERT로 |
