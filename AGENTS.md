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
| `docs/roadmap.md` | 단계·일정·진행 상태 | **메인 개발 세션만** 갱신 |
| `docs/DESIGN.md` | 디자인 토큰·컴포넌트 규칙 | UI 작업 전 필독 |
| `docs/api.md` | REST/WebSocket 명세 | 엔드포인트 변경 시 같은 PR에서 갱신 |
| `docs/design-refs/` | Stitch 시안 이미지 | 참고용 |

## 2. 스택
- `backend/` Rust · Axum · SQLx · PostgreSQL · livekit-api
- `frontend/` React · Vite · TypeScript · Tailwind · livekit-client
- `deploy/` Docker Compose · nginx · certbot · Prometheus · Loki · Grafana · Alloy
- 스트리밍 LiveKit Cloud / 결제 토스페이먼츠 테스트 모드

## 3. 명령어
```bash
docker compose -f deploy/compose.dev.yml up -d   # 로컬 Postgres
cd backend && cargo run                           # 백엔드
cd backend && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd backend && cargo sqlx prepare                  # 쿼리 변경 시 .sqlx 커밋
cd frontend && npm run dev                        # 프론트
cd frontend && npm run lint && npm run typecheck && npm run build
```
마이그레이션: `sqlx migrate add <이름>`, 적용은 서버 시작 시 자동.

## 4. 개발 우선순위
앞 단계의 완료 기준을 통과하기 전에 뒤 단계로 가지 않는다.
필수 기능 → 댓글·밴·클러치 버튼 → 도네이션·예치금 → 녹화 → 팔로우 → Clutch Time 대기열·클립

## 5. 코드 작성 원칙 (Ponytail: 게으른 시니어 개발자)
> 출처: [DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail) `AGENTS.md` (MIT) 한국어 번역. 이 프로젝트에는 Ponytail 플러그인이 프로젝트 범위로 설치되어 있다(`.claude/settings.json`). 플러그인이 없는 환경에서도 이 절을 따른다.

최고의 코드는 쓰지 않은 코드다. 문제 전체를 가장 적은 새 코드로 푼다. **모든 답변은 1~2줄로 끝낸다: 건너뛰었거나 확인하지 않은 것, 사용자가 알아야 할 위험.**

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
- Clutch 게이지·대기열 규칙

**테스트를 쓰지 않는 곳**
- UI 마크업, 문구, 스타일, 단순 CRUD 조회
- 구현 세부를 고정하는 테스트(내부 함수 호출 순서 등)
- 브라우저 E2E 자동화 (송출·시청은 README 수동 체크리스트로 확인)

**실행 규칙**
- 작업 중에는 **바꾼 영역의 테스트만** 돌린다: `cargo test --test <시나리오파일>` 또는 `cargo test <모듈명>`.
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
- `.github/pull_request_template.md` 채우기: 요약 / 처리 사항 / 테스트 / 충족 스펙 항목.
- **CI 통과 시 Squash merge 후 브랜치 삭제.** 별도 리뷰 없음.
- 예외: `auth` `donation` `wallet` 범위 PR은 머지 전 `/ponytail-review branch` 1회.

**도구 설정 (`.claude/settings.json`, 이 프로젝트에만 적용)**
- Ponytail 플러그인 활성화.
- oh-my-claudecode의 매 도구 호출마다 붙는 "테스트하라/계속하라" 훅과 키워드 자동 모드 전환은 `OMC_SKIP_HOOKS`로 끈다(§7 테스트 규칙과 충돌하기 때문).

## 9. 세션·병렬 작업 규칙
구조: **상주 채팅 세션**(사용자와 대화) → **메인 개발 세션**(이 문서와 roadmap에 따라 작업 분해·머지) → **작업 세션**(작업 1개만 수행 후 PR).

- 작업 세션 1개 = 작업 1개 = 브랜치 1개 = **git worktree 1개**. 같은 폴더에서 두 세션이 쓰지 않는다.
- **동시 작업 세션은 최대 3개** (Rust 빌드 캐시가 worktree마다 수 GB, CPU 경쟁).
- 11/9~10 기반 작업(골격·스키마·인증·레이아웃·배포)은 **직렬**로 한다. 병렬은 그 이후.
- 병렬로 나눌 때는 **파일이 겹치지 않는 단위**로 나눈다 (예: 백엔드 chat 모듈 / 프론트 시청 화면).
- 프론트·백엔드가 같은 기능을 나눠 맡으면 **`docs/api.md` 계약을 먼저 머지**하고 시작한다.
- 충돌 위험 파일은 **메인 개발 세션만** 고친다: `docs/roadmap.md`, 라우터 등록부(`backend/src/routes/mod.rs`), `Cargo.toml`/`package.json` 의존성 추가는 PR에 명시.
- 마이그레이션: 동시에 두 작업이 마이그레이션을 추가하지 않는다. 필요하면 메인 세션이 순서를 정한다.
- 포트는 worktree마다 `.env`의 `BACKEND_PORT`·`VITE_PORT`로 분리한다. 로컬 dev DB 이름도 worktree별로 나눈다.
- PR 전 `main`으로 rebase. 머지는 메인 개발 세션이 의존 순서대로 한다.
- 작업 세션은 컨텍스트 40% 전후에서 정리하거나 작업을 끝낸다. 한 세션에 작업을 이어 붙이지 않는다.

## 10. 완료 기준 (PR 올리기 전)
- [ ] 백엔드·프론트 검사 명령 통과, 전체 테스트 1회 통과
- [ ] 위험 영역이면 시나리오 테스트 추가
- [ ] `docs/api.md`, README 기능 확인 절차 갱신 (해당 시)
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
