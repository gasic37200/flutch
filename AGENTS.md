# FLUTCH — Agent Guide

게임·버추얼 중심 라이브 스트리밍 플랫폼. 공모전 제출작 (마감 **2026-11-19 23:59 KST**).
슬로건: "Feel the Moment. Live on FLUTCH."

## 1. 문서 지도 (작업 전 해당 문서를 먼저 읽는다)
| 문서 | 내용 | 규칙 |
|---|---|---|
| `docs/spec.md` | 공모전 원문 요구사항 | 모든 판단에 우선. 수정 금지 |
| `docs/decisions.md` | 확정 정책·기술 결정과 이유 | 바꾸려면 사용자 승인 후 이 문서부터 수정 |
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

## 5. 코드 작성 원칙 (최소 코드)
- 요구사항을 만족하는 **가장 짧은 올바른 코드**를 쓴다. 쓰기 전에 이미 있는 코드·표준 라이브러리·기존 의존성으로 되는지 먼저 본다.
- 쓰지 않을 확장 포인트, 추상화, 설정, 범용 유틸을 미리 만들지 않는다.
- 새 의존성은 직접 구현이 50줄을 넘거나 보안·암호·프로토콜 영역일 때만 추가한다.
- **줄이지 않는 것**: 서버 권한 검증, 입력 검증(신뢰 경계), 돈·원장 정합성, 데이터 손실 방지, 에러 처리, 접근성.

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
- 예외: `auth` `donation` `wallet` 범위 PR은 머지 전 `/code-review low` 1회.

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
