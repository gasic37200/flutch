# FLUTCH — Agent Guide

게임·버추얼 중심 라이브 스트리밍 플랫폼. 공모전 제출작 (마감 **2026-11-19 23:59 KST**).
슬로건: "Feel the Moment. Live on FLUTCH."

## 1. 문서 지도 (작업 전 해당 문서를 먼저 읽는다)
| 문서 | 내용 | 규칙 |
|---|---|---|
| `docs/spec.md` | 공모전 원문 요구사항 | 모든 판단에 우선. 수정 금지 |
| `docs/decisions.md` | 확정 정책·기술 결정과 이유 | 바꾸려면 사용자 승인 후 이 문서부터 수정 |
| `docs/architecture.md` | 전체 구성·흐름·코드 구조·데이터 모델·백그라운드 작업 | 새 모듈·테이블 추가 전 필독 |
| `docs/features/` | 서비스별 세부 정책·시나리오 테스트 이름 | **해당 기능 작업 전 필독**. decisions와 다르면 이쪽 우선 |
| `docs/backlog.md` | 범위 밖 부가 기능 | roadmap 완료 전 착수 금지 |
| `docs/roadmap.md` | 단계·일정·진행 상태 | **메인 세션만** 갱신 |
| `docs/DESIGN.md` | 디자인 토큰·컴포넌트 규칙 | UI 작업 전 필독 |
| `docs/api.md` | REST/WebSocket 명세 | 엔드포인트 변경 시 같은 PR에서 갱신 |
| `docs/conventions.md` | 안티패턴, Rust·Axum·SQLx·React 주의점, 코드 관례 | **코드 작성 전 필독** (모든 서브에이전트) |
| `docs/design-refs/` | Stitch 시안 이미지 | 참고용 |
| `docs/troubleshooting/` | 작업 중 겪은 문제와 해결 기록 | 같은 영역 작업 전 확인, 서브에이전트가 기록 |
| `docs/worklog.md` | 서브에이전트별 소요 시간·토큰·결과 | 메인 세션만 기록 (§9-7) |
| [Notion: Flutch 기능 및 트러블슈팅](https://app.notion.com/p/Flutch-41349c16438d8083a407de3fbedf0830) | 사람이 보는 요약본(기능 완료·트러블슈팅·하루 요약) | 메인 세션만, Notion 커넥터로 갱신 (§9-11) |

## 2. 스택
- `backend/` Rust · Axum · SQLx · PostgreSQL · livekit-api
- `frontend/` React · Vite · TypeScript · Tailwind · livekit-client
- `deploy/` Docker Compose · nginx · certbot · Prometheus · Loki · Grafana · Alloy
- 스트리밍 LiveKit Cloud / 결제 토스페이먼츠 테스트 모드

## 3. 명령어
> **개발·빌드·테스트는 WSL2 Ubuntu(`~/flutch`)에서 한다.** Windows는 스마트 앱 컨트롤이 빌드한 실행 파일을 막아 `cargo run`이 안 된다. 앱 세션은 SSH `localhost:2222`(사용자 `dbsrh`)로 연다. 운영 서버 SSH 키는 `/mnt/c/Users/dbsrh/.ssh/flutch_oci`.
```bash
docker compose --env-file .env -f deploy/compose.dev.yml up -d   # 로컬 Postgres(호스트 5433)·LiveKit, 모든 worktree 공유
cd backend && cargo run                           # 백엔드
cd backend && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd backend && cargo sqlx prepare -- --all-targets # 쿼리 변경 시 .sqlx 커밋(옵션 없으면 테스트 전용 쿼리 캐시가 지워져 CI 실패, T20b)
cd frontend && npm run dev                        # 프론트
cd frontend && npm run lint && npm run typecheck && npm run build
```
마이그레이션: `sqlx migrate add <이름>`, 적용은 서버 시작 시 자동.

## 4. 개발 우선순위
앞 단계의 완료 기준을 통과하기 전에 뒤 단계로 가지 않는다.
필수 기능 → 댓글·밴·클러치 버튼 → 도네이션·예치금 → 녹화 → 팔로우·카카오 → SUPER CLUTCH 회전·클립

## 5. 코드 작성 원칙 (Ponytail: 게으른 시니어 개발자)
> 출처: [DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail) `AGENTS.md` (MIT) 한국어 번역. 이 프로젝트에는 Ponytail 플러그인이 프로젝트 범위로 설치되어 있다(`.claude/settings.json`). 플러그인이 없는 환경에서도 이 절을 따른다.

최고의 코드는 쓰지 않은 코드다. 문제 전체를 가장 적은 새 코드로 푼다. **모든 답변은 1~2줄로 끝낸다: 건너뛰었거나 확인하지 않은 것, 사용자가 알아야 할 위험.** (예외: 서브에이전트의 §9-5 요약 보고는 양식의 모든 필드를 쓴다.)

**쓰기 전에**
작업과 그 작업이 건드리는 코드를 읽는다. 변경이 닿아야 하는 모든 곳(호출처, 테스트, 픽스처, 설정, export)을 나열한다. 변경이 사용자에게 깨뜨릴 수 있는 것(파괴·노출될 데이터, 동작을 멈출 호출처)을 확인한다. 그것이 범위다. 추가 기능은 범위가 아니다.

**가장 작은 완전한 변경** — 완전히 동작하는 첫 번째 선택지를 고른다:
1. 꼭 있어야 하나? 아무도 요청하지 않은 기능·옵션·유연성은 건너뛰고 한 줄로 밝힌다.
2. 이미 이 코드베이스에 있나(헬퍼, 컴포넌트, 서비스, 패턴)? 주변 코드와 같은 방식으로 쓴다.
3. 표준 라이브러리나 플랫폼 기능? 쓴다. 단, 프로젝트 자체 구현이 있으면 그것이 우선.
4. 설치된 의존성? 쓴다. 몇 줄 때문에 의존성을 추가하지 않는다.
5. 읽는 사람이 한눈에 아는 한 줄로 되나? 한 줄.
6. 그 외: 동작하는 최소 코드.

- 해법에는 게으르되 변경 자체에는 게으르지 않는다: 변경으로 깨지는 호출처·테스트·픽스처까지 끝낸다.
- 요청받지 않은 추상화, 래퍼, 타입 변환, 옵션, 설정, 보일러플레이트, "나중을 위한" 코드 금지. 삭제가 추가보다 낫다. 코드베이스의 기존 구조(계층, 인터페이스, 관례)를 유지한다.
- 닿아야 할 곳을 다 안 뒤에는 가장 짧은 동작하는 diff가 이긴다. 해독이 필요한 한 줄은 짧은 게 아니다.
- 주석은 코드가 보여줄 수 없는 "왜"만, 한 줄로.
- 버그 수정: 고치기 전에 그 함수의 모든 호출처를 grep하고, 공통 코드에서 근본 원인을 한 번에 고친다.
- 옮기거나 합치는 코드는 에러 처리와 검증을 그대로 유지한다.
- 크기가 같은 선택지 중에서는 경계 조건에서 올바른 쪽을 고른다.
- 새로운 비자명 로직(분기, 반복, 파서, 돈, 보안)은 작은 테스트 하나를 남긴다(§7 테스트 규칙 우선). 사소한 변경은 테스트 불필요.
- 알려진 한계가 있는 지름길은 주석으로 남긴다: `shortcut: <한계>, <언제 개선할지>`

**절대 줄이지 않는 것**: 신뢰 경계의 입력 검증, 서버 권한 검증, 돈·원장 정합성, 데이터 손실을 막는 에러 처리, 보안, 접근성, 사용자가 요청한 것.

**Ponytail 명령 사용 시점**
- `auth` `donation` `wallet` 범위 PR 머지 전: `/ponytail-review branch`
- 필수 기능 완료 시(11/13)와 제출 전(11/18): `/ponytail-audit` 1회씩, 결과의 must fix를 roadmap에 반영
- 제출일(11/19): `/ponytail-debt`로 `shortcut:` 목록을 모아 README "알려진 한계"에 반영
- 강도는 기본 `full`. `ultra` 사용 금지.

## 6. 반드시 지킬 규칙
- 권한은 **항상 서버에서** 검증한다. 화면에서 숨기는 것은 권한 처리가 아니다.
- 비트 원장은 **추가만** 한다. 수정·삭제 금지. 잔액 차감은 조건부 `UPDATE ... WHERE balance >= $amount`로만.
- 결제 확정은 멱등하다 (`paymentKey`·`orderId` 유니크).
- 방송 상태 전이는 `READY → LIVE → ENDED`만 허용. 그 외 전이는 서버가 409로 거절.
- 익명 후원자의 실제 닉네임은 **방송 소유자 요청에만** 내려준다.
- 색·간격·글꼴은 **디자인 토큰만** 쓴다. hex 값 직접 사용 금지.
- 화면 컴포넌트는 표현만, 로직은 `hooks/`에 둔다 (디자인 교체 대비).
- 사용자에게 보이는 문구는 한국어.
- 외부 코드·에셋·폰트를 가져오면 README 출처 목록에 추가한다.

## 7. 테스트 규칙
테스트는 **위험한 곳에만, 필요한 만큼만** 돌린다.

**테스트를 반드시 쓰는 곳** (시나리오 테스트, `backend/tests/scenarios/`)
- 권한 규칙 (비로그인 / 소유자 / 다른 사용자 / 차단 사용자)
- 방송 상태 전이
- 돈: 결제 확정, 원장, 잔액 차감, 멱등성(같은 승인 통지 반복)
- Clutch 게이지·회전 규칙

**테스트를 쓰지 않는 곳**
- UI 마크업, 문구, 스타일, 단순 CRUD 조회
- 구현 세부를 고정하는 테스트(내부 함수 호출 순서 등)
- 브라우저 E2E 자동화 (송출·시청은 README 수동 체크리스트로 확인)

**실행 규칙**
- 작업 중에는 **바꾼 영역의 테스트만** 돌린다: `cargo test --test scenarios <시나리오 이름>` 또는 `cargo test <모듈명>`.
- 전체 테스트(`cargo test`)는 **PR 올리기 직전 1회**. 그 외 전체 실행은 CI가 한다.
- 같은 실패로 **3번 고쳐도 안 되면 멈추고** 원인 가설과 함께 보고한다. 테스트를 끄거나 `#[ignore]`로 우회하지 않는다.
- DB 테스트는 `#[sqlx::test]`로 테스트마다 격리된 DB를 쓴다 (병렬 세션끼리 충돌 없음).
- 시나리오 이름은 README "기능 확인" 절차와 1:1로 맞춘다.

## 8. Git 규칙
**브랜치** (GitHub Flow)
- `main`: 항상 배포 가능. 머지 시 자동 배포. **직접 push 금지**.
- 작업 브랜치: `feat/…` `fix/…` `refactor/…` `test/…` `docs/…` `chore/…` `ci/…` (예: `feat/broadcast-crud`)

**커밋** (Conventional Commits, 타입·범위 영어 + 설명 한국어)
```
feat(broadcast): 송출 시작·종료 API 추가

- POST/DELETE /api/v1/broadcasts/{id}/live 추가
- 소유자에게만 LiveKit publish 토큰 발급
- 종료 시 LiveKit 방을 닫아 송출 중단
```
- 타입: `feat` `fix` `refactor` `test` `docs` `chore` `ci` `style` `perf`
- 범위: `auth` `broadcast` `live` `chat` `moderation` `donation` `wallet` `clutch` `recording` `follow` `ui` `infra` `docs`
- 제목은 50자 안팎, 마침표 없음. 본문은 처리 사항 bullet.

**PR**
- 기능 1개 = PR 1개. 제목은 커밋 제목과 같은 형식 (Squash 시 그대로 커밋이 됨).
- 메인 세션은 `gh pr merge <n> --squash --subject "<PR 제목> (#<n>)"`로 머지한다(GitHub이 브랜치 첫 커밋 제목을 squash 제목으로 쓰는 경우가 있음 — T22에서 `wip(…)`이 main에 들어감). 서브에이전트는 커밋 제목에 `wip`을 쓰지 않는다.
- `.github/pull_request_template.md` 채우기: 요약 / 처리 사항 / 테스트 / 충족 스펙 항목.
- **CI 통과 + 메인 세션의 보고서·docs 변경 판단(§9-2) 후 Squash merge, 브랜치 삭제.** 코드 리뷰는 별도로 하지 않음.
- 예외: `auth` `donation` `wallet` 범위 PR은 머지 전 `/ponytail-review branch` 1회.

**도구 설정 (`.claude/settings.json`, 이 프로젝트에만 적용)**
- Ponytail 플러그인 활성화.
- oh-my-claudecode의 매 도구 호출마다 붙는 "테스트하라/계속하라" 훅과 키워드 자동 모드 전환은 `OMC_SKIP_HOOKS`로 끈다(§7 테스트 규칙과 충돌하기 때문).

## 9. 세션 운영 규칙

### 9-1. 역할
| 세션 | 하는 일 | 하지 않는 일 |
|---|---|---|
| **메인 세션** (사용자가 대화하는 유일한 세션) | 사용자와 대화, 결정을 `docs/`에 반영, `docs/` 전체를 숙지한 상태로 roadmap task를 서브에이전트에게 할당, 결과 판단·후속 할당·머지·roadmap·worklog 갱신 | 큰 기능 직접 구현(작은 수정·충돌 해결·문서만) |
| **서브에이전트** | task 1개 구현 → 트러블슈팅 기록 → PR → 요약 보고 | 다른 task 손대기, 머지, roadmap 수정 |

### 9-2. 메인 세션 루프
1. **시작 시 숙지**: `docs/spec.md` → `decisions.md` → `architecture.md` → `features/` → `DESIGN.md` → `api.md` → `roadmap.md` → `troubleshooting/README.md`.
2. **할당**: roadmap에서 선행이 모두 ✅인 task를 고른다. 서브에이전트(스레드)는 **Agent 도구 백그라운드 실행**으로 띄우고, worktree는 서브에이전트가 **WSL에서 직접** 만든다(`git worktree add ~/flutch-wt/<task> -b <브랜치> origin/main`, 파일은 UNC `\\wsl.localhost\Ubuntu-24.04\home\dbsrh\flutch-wt\<task>`, 명령은 SSH `localhost:2222`). 메인 세션이 Windows에서 돌면 `isolation: "worktree"`는 빌드할 수 없는 Windows 쪽에 worktree를 만들기 때문. 지시서는 §9-4 양식. 띄운 시각을 기록해 둔다.
   - 서브에이전트가 도는 동안에도 메인 세션은 사용자와 대화할 수 있다. 사용자 질문에 답하느라 할당을 멈추지 않는다.
3. **보고 판단** (§9-5 보고서 기준):
   - "위험·건너뛴 것"이 있으면 → 판단해서 **후속 서브에이전트에게 다시 할당**(같은 PR 브랜치에 이어서, 또는 새 task). 사용자 결정이 필요한 것만 §9-6으로.
   - PR의 **docs 변경분만 따로 검토**: decisions·features·architecture·api와 모순이 없고 사실이면 승인하고 **변경된 문서를 다시 읽어 숙지**한다. 틀리면 서브에이전트에게 수정 요청.
   - `auth` `donation` `wallet` 범위면 `gh pr checkout <번호>`로 PR 브랜치를 받은 뒤 `/ponytail-review branch` 1회, 끝나면 main으로 돌아온다.
4. **머지**: CI 통과 + 위 판단 통과 → Squash merge → roadmap 상태와 **`docs/worklog.md`**(§9-7) 갱신을 묶어 docs PR.
5. **배포 확인**: 머지 후 자동 배포가 끝나면 `GET /api/v1/health` 확인. 실패하면 **즉시 revert PR을 올려 머지**하고 원인 task를 다시 할당.
6. **컨텍스트 관리**: task를 머지할 때마다 roadmap에 상태가 남아 있으므로, 컨텍스트가 길어지면 `/compact` 후 1번(숙지)부터 다시.

### 9-3. 병렬 규칙
- 서브에이전트 1개 = task 1개 = 브랜치 1개 = worktree 1개. **동시 최대 3개**(Rust 빌드 캐시가 worktree마다 수 GB, CPU 경쟁).
- 기반 단계는 T01 → T02 → T02b → T03 → T04 **직렬**, T05·T07은 T01 직후부터 병렬. 그 외 병렬은 roadmap에 "(병렬)" 표시된 task끼리만.
- 파일이 겹치지 않는 단위로만 병렬. 프론트·백엔드가 같은 기능을 나누면 `docs/api.md` 계약을 먼저 머지.
- 공용 파일(`backend/src/routes/mod.rs`, `main.rs`, `config.rs`, `scheduler.rs`, `error.rs`, `frontend/src/router.tsx`)은 T01이 만든 골격대로 **자기 도메인 부분만** 채운다(라우터는 미리 연결돼 있음). 골격 밖 수정이 필요하면 보고서에 요청한다. `docs/roadmap.md`·`docs/worklog.md`는 메인 세션만.
- 의존성 추가(`Cargo.toml`·`package.json`)는 보고서에 명시. lock 파일 rebase 충돌은 메인 세션이 해결한다.
- 마이그레이션은 동시에 한 task만 추가. 포트는 worktree마다 `.env`의 `BACKEND_PORT`·`VITE_PORT`로 분리.
- PR 전 `main`으로 rebase. 머지는 메인 세션이 의존 순서대로.
- **처리 순서**: roadmap 표 순서 + 선행 관계. 선행이 끝난 task가 여러 개면 표에서 위에 있는 것부터, 병렬 가능한 것은 최대 3개 동시에. 하나가 끝나면 즉시 다음 가능한 task를 채운다(대기 줄이기).
- **빌드 병목 대책**: 로컬은 `sccache`로 worktree 간 컴파일 캐시 공유(`RUSTC_WRAPPER=sccache`), CI는 `Swatinem/rust-cache`. `CARGO_TARGET_DIR` 공유는 병렬 빌드가 잠금으로 줄을 서게 되므로 쓰지 않는다.

### 9-4. 작업 지시서 양식 (메인 → 서브에이전트)
```
[Txx] <task 이름>
- 읽을 문서: AGENTS.md, docs/conventions.md, docs/features/<nn>.md, docs/architecture.md §<n>, (해당 시) docs/api.md·DESIGN.md, **화면 task는 `docs/design-refs/boards/`의 해당 보드(README 대응표)**
- 모델·사고: <roadmap의 모델·사고 칸 그대로>
- 건드릴 범위: <폴더/파일>
- 건드리지 말 것: docs/roadmap.md·worklog.md, 공용 파일의 다른 도메인 부분(골격 밖 수정은 보고서에 요청), 다른 task 영역
- 완료 기준: <features 문서의 시나리오 테스트 이름 / 화면 확인 항목>
- 브랜치: <type>/<이름> (push는 이 브랜치만, main 직접 push 금지)
- 포트·DB: BACKEND_PORT=<…>, VITE_PORT=<…>, DATABASE_URL=…/flutch_<task>
- 임시 파일: `/tmp/<task>/` 아래에만 (WSL `/tmp`는 병렬 서브에이전트끼리 공유 — 같은 이름 스크립트가 덮어써져 다른 worktree를 건드린 사고, T04)
- Browser 확인: 창을 다른 서브에이전트와 공유하므로 `tabs_create`로 새 탭을 열어 그 tabId로만 작업하고 끝나면 닫는다(T11에서 다른 task가 탭 주소를 바꾼 일)
- 로그인 쿠키는 포트가 달라도 `localhost`끼리 공유된다(T09c) → Browser는 `http://<task>.localhost:<VITE_PORT>`로 열고 `.env`의 `ALLOWED_ORIGINS`도 같은 주소로
- 이전 시도·후속 할당이면: <이전 보고서의 위험·건너뛴 것>
```

### 9-5. 서브에이전트 마무리 순서
1. 구현 + 해당 시나리오 테스트 (§7).
2. **트러블슈팅 기록**: 작업 중 막혔던 것·예상과 달랐던 것·시행착오가 있었으면 `docs/troubleshooting/`에 기록(양식은 그 폴더 README). 없으면 보고서에 "트러블슈팅 없음".
3. **docs 갱신이 필요하면 같은 PR에 포함**: api.md 변경, features 정책의 구현상 보정, architecture 변경 등. 확정 결정(decisions)을 바꿔야 하면 바꾸지 말고 보고서에 "결정 변경 필요"로 올린다.
4. §10 완료 기준 확인 → PR 생성.
5. **요약 보고**(최종 응답, 메인 세션에게). 메인 세션의 컨텍스트를 아끼기 위한 요약이지만 **아래 필드는 생략·축약 금지**. 상세 내용은 보고서가 아니라 PR 본문·트러블슈팅 파일에 두고, 보고서는 그 위치를 가리킨다.
```
[Txx] 완료 | 부분 완료 | 실패  ·  PR: <링크>  ·  브랜치: <이름>
- 처리 사항: <3~5줄, 완료 기준 항목별 충족 여부>
- 계약 변경: <다른 task가 의존하는 것: API 경로·요청/응답 필드, DB 테이블·컬럼, WebSocket 이벤트, 환경변수, 공개 함수. 없으면 "없음">
- 테스트: <돌린 명령과 결과 / 추가한 시나리오 이름>
- docs 변경: <파일 — 한 줄 이유. 없으면 없음>
- 트러블슈팅: <파일 경로 또는 없음>
- 의존성 추가: <crate·패키지 이름과 이유. 없으면 없음>
- 가정한 것: <문서에 없어서 가정하고 진행한 결정. 없으면 없음>
- 위험·건너뛴 것·확인 못 한 것: <각각 이유와 함께. 없으면 없음>
- 결정 변경 필요: <없음 또는 내용과 추천안>
- 다음 task 영향: <후속 task가 알아야 할 것. 없으면 없음>
```
- "계약 변경"과 "위험·건너뛴 것"이 비어 있지 않으면 메인 세션은 PR diff에서 해당 부분을 직접 확인한다.
- "가정한 것"은 메인 세션이 roadmap "사용자 확인 대기" 표로 옮긴다.

### 9-6. 사용자에게 묻는 경우 (멈추지 않는 것이 원칙)
사용자는 하루 종일 자리를 비운다. **질문 때문에 전체 작업이 멈추면 안 된다.**

1. **먼저 docs로 판단한다.** spec → features → decisions → architecture 순으로 근거를 찾고, 있으면 묻지 않는다.
2. **되돌릴 수 있는 결정은 가정하고 진행한다.** 문서에 없고 되돌리기 쉬운 것(문구, 수치, UI 배치, 내부 구조)은 가장 보수적이고 스펙에 맞는 선택으로 진행하고, `docs/roadmap.md`의 **"사용자 확인 대기"** 표에 `가정한 내용 / 이유 / 바꾸면 영향`을 적는다. 사용자가 나중에 다르게 답하면 그때 수정 task를 만든다.
3. **멈추고 묻는 것은 아래만.** 이때도 해당 task만 멈추고 다른 독립 task는 계속한다.
   - §11 승인 항목(외부 계정·비용, 기능 빼기, 저장소 공개, 제출)
   - 데이터·돈이 걸려 되돌리기 어려운 결정(원장 구조, 권한 규칙, 운영 DB 스키마의 파괴적 변경)
   - 사용자만 가진 것(키 입력, 서버·도메인 준비)
4. **질문은 모아서 한 번에.** 같은 시점의 질문은 한 메시지로 묶고, 각 질문에 추천안을 함께 쓴다. 질문을 올리면 **PushNotification으로 알린다**(지연 로드 도구라 처음엔 ToolSearch `select:PushNotification`으로 불러온다).
5. 사용자 답은 메인 세션이 바로 `docs/`에 반영(docs PR)하고, 진행 중인 서브에이전트에 영향이 있으면 지시서를 갱신해 후속 할당한다.

### 9-7. 작업 기록 (worklog)
서브에이전트가 끝나면 메인 세션이 `docs/worklog.md`에 한 줄을 추가한다. **시각은 모두 한국시간(KST)**으로 `YYYY-MM-DD HH:MM` 형식(`TZ=Asia/Seoul date '+%F %H:%M'`). 띄운 시각은 Agent 호출 직전에, 끝난 시각은 완료 알림을 받은 직후에 잰다. 서브에이전트는 자기 토큰 사용량을 정확히 모르므로 **Agent 도구 결과에 표시되는 사용량·소요 시간**을 메인 세션이 옮겨 적는다.
| 날짜 | task | 결과(완료·부분·실패) | 시작 → 종료 | 소요 시간 | 토큰 | 도구 호출 | PR | 메모 |
|---|---|---|---|---|---|---|---|---|
- 후속 할당(같은 task 재시도)은 별도 줄, task 칸에 `T08 (2차)`.
- 하루가 끝나면 마지막 줄에 그날 합계(task 수, 총 시간, 총 토큰).
- 메인 세션 자체의 사용량은 Ponytail band(비용·토큰·시간)로 확인하고 하루 합계 줄에 함께 적는다.

### 9-8. 검증 범위
- 서브에이전트는 시나리오 테스트·검사 명령으로 백엔드를 검증하고, 화면은 Browser 미리보기로 **레이아웃과 콘솔 에러만** 확인한다.
- 화면 공유 소리, 휴대폰 카메라, 다른 기기 시청은 AI가 확인할 수 없다 → roadmap "사용자 준비물"의 **U1 수동 확인**으로 검증하고, 메인 세션은 U1 전에는 T15를 완료 처리하지 않는다.
- 로컬에서 송출·웹훅은 compose.dev의 로컬 LiveKit으로 확인한다. 녹화(Egress)와 토스 웹훅은 배포 환경에서만 실제 확인된다.

### 9-9. 모델과 사고 단계
| 역할 | 모델 | 사고 단계(`effort`) |
|---|---|---|
| 메인 세션 | Opus 5.5 | high |
| 위험 task (roadmap "Opus · high": 골격·스키마·인증·LiveKit·WebSocket 필터링·원장·결제·후원·유료 클러치·회전) | `model: "opus"` | `effort: "high"` |
| API 계약 문서(T02b) | `model: "opus"` | `effort: "medium"` |
| 나머지 (화면, CRUD, 채팅·모더레이션, 녹화, 팔로우, README, 모니터링 등) | `model: "sonnet"` | `effort: "medium"` |
- 같은 task를 다시 맡길 때(2차)는 사고 단계를 한 단계 올린다(medium → high → xhigh). Sonnet으로 두 번 실패하면 Opus로 바꾼다.
- `max`는 쓰지 않는다(비용 대비 효과 낮음). `low`는 오타·문구 수정 같은 단순 변경에만.
- worklog에 모델·사고 단계를 메모 칸에 적어 두고, 하루 단위로 비용 대비 결과를 보고 배분을 조정한다.
- Ponytail 강도는 모든 세션 `full`. 화면 task 결과가 시안보다 지나치게 단순하면 그 task만 `lite`로 다시 맡긴다.

### 9-10. 시간 표기
- 사용자에게 보이는 모든 시각(보고, worklog, roadmap, PR 본문, 트러블슈팅 날짜)은 **KST**. 마감도 "2026-11-19 23:59 KST" 기준으로 남은 시간을 계산한다.
- 개발 환경(WSL)과 운영 서버의 OS 시간대는 `Asia/Seoul`로 설정돼 있다. 코드·DB의 시간 규칙은 `docs/architecture.md` §7-1.

### 9-11. Notion 요약 기록
- 원본은 저장소(`docs/troubleshooting/`, `docs/worklog.md`)다. Notion 페이지는 사용자가 보기 쉬운 **요약본**이다. 서브에이전트는 Notion에 쓰지 않는다(동시에 쓰면 섞인다).
- 메인 세션이 Notion 커넥터로 갱신한다:
  - **task 머지 때마다**: "기능 완료 내역"에 `날짜·시각(KST) · Txx · 한 줄 요약 · PR 링크` 한 줄 추가. 그 PR에 트러블슈팅 파일이 있으면 "트러블슈팅"에 소제목 하나로 요약(증상/원인/해결/재발 방지/관련 task·PR).
  - **하루 끝**: "하루 요약"에 완료 task 수, 총 소요 시간, 총 토큰, 남은 일정.
- 비밀값·서버 키·토큰은 Notion에 적지 않는다. Notion 커넥터를 쓸 수 없으면 건너뛰고 worklog 메모에 "Notion 미반영"이라 적은 뒤, 다음에 쓸 수 있을 때 한꺼번에 반영한다(작업을 멈추지 않는다).

## 10. 완료 기준 (PR 올리기 전)
- [ ] 백엔드·프론트 검사 명령 통과, 전체 테스트 1회 통과
- [ ] 위험 영역이면 시나리오 테스트 추가
- [ ] `docs/api.md`, README 기능 확인 절차 갱신 (해당 시)
- [ ] 시행착오가 있었으면 `docs/troubleshooting/` 기록
- [ ] 비밀값·`.env` 미포함

## 11. 사용자 승인이 필요한 일 (그 외에는 자율 진행)
- 외부 계정 생성, 실제 결제, 비용이 발생하는 작업
- 일정 부족으로 기능을 빼는 결정
- 저장소 공개 전환, 제출
- `docs/decisions.md` 확정 결정 변경

## 12. 금지
- `main` 직접 push, force push
- `.env`·키·토큰 커밋 (`.env.example`만 커밋)
- 테스트 끄기·skip으로 CI 통과시키기
- `decisions.md`에 없는 기능 추가
