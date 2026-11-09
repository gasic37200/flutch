# FLUTCH — 심장이 뛰는 순간, 플러치.

FLUTCH(플러치)는 게임과 버추얼 콘텐츠를 중심으로 스트리머와 시청자가 함께 즐기는 라이브 스트리밍 플랫폼입니다.
FLUTTER의 두근거림과 CLUTCH의 짜릿함을 결합한 이름처럼, 라이브 방송에서 생기는 모든 순간의 감정과 에너지를 중요하게 생각합니다.

**Feel the Moment. Live on FLUTCH.**

> 🚧 개발 중입니다. 아래 항목은 제출 전에 채워집니다.

## 서비스 주소
- https://flutch.kro.kr (준비 중)

## 테스트 계정
- 로그인 화면의 **"테스트 계정으로 바로 로그인"** 버튼 (준비 중)

## 실행 방법

### 개발
> Windows에서는 스마트 앱 컨트롤이 빌드한 실행 파일을 막으므로 **WSL2 Ubuntu**(`~/flutch`)에서 빌드·실행합니다. Docker는 Docker Desktop의 WSL 연동을 씁니다.

**1. 도구 (한 번만)**
- Rust stable, Node 22, Docker
- sqlx-cli: `cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres,rustls --locked`
- sccache(선택, worktree 간 컴파일 캐시 공유): `cargo install sccache --locked` 후 `~/.bashrc`에 `export RUSTC_WRAPPER=sccache`
  (`.cargo/config.toml`에 넣지 않습니다. CI·sccache 없는 환경이 깨집니다.)

**2. 환경변수**
```bash
cp .env.example .env   # 저장소 루트. 백엔드·Vite·compose가 모두 이 파일을 읽습니다
```
worktree를 여러 개 동시에 쓸 때는 worktree마다 `.env`의 `BACKEND_PORT`·`VITE_PORT`·`ALLOWED_ORIGINS`(Vite 주소)·`DATABASE_URL`의 DB 이름(`flutch_<task>`)을 다르게 둡니다. 5432 포트를 다른 Postgres가 쓰고 있으면 `POSTGRES_PORT`와 `DATABASE_URL`의 포트를 같이 바꿉니다.

**3. 실행**
```bash
docker compose --env-file .env -f deploy/compose.dev.yml up -d   # Postgres 16 + 로컬 LiveKit(--dev)
(cd backend && sqlx database create && cargo run)                # 마이그레이션은 시작 시 자동 적용
curl localhost:8080/api/v1/health                                # 포트는 BACKEND_PORT. {"status":"ok"}
(cd frontend && npm install && npm run dev)                      # 다른 터미널. /api는 백엔드로 프록시(WebSocket 포함)
```
- compose 컨테이너는 모든 worktree가 함께 씁니다. LiveKit 웹훅은 compose를 마지막으로 띄운 `.env`의 `BACKEND_PORT`로 갑니다(다른 worktree로 바꾸려면 그 worktree에서 다시 `up -d`).
- 로컬 LiveKit: `ws://localhost:7880`, 키 `devkey` / 시크릿 `secret`.

**4. 검사·테스트**
```bash
cd backend && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd backend && cargo test                          # 시나리오 테스트(실제 Postgres) + ts-rs 타입 생성
cd backend && cargo test --test scenarios health  # 시나리오 일부만
cd backend && cargo sqlx prepare                  # query! 를 바꿨으면 .sqlx/ 갱신 후 커밋
cd frontend && npm run lint && npm run typecheck && npm run build
```
- API 타입은 `cargo test`가 `frontend/src/api/generated/`에 생성합니다. 바뀌면 함께 커밋합니다(CI가 비교).

## 기능 확인
준비 중

## 문서
- [공모전 요구사항](docs/spec.md)
- [설계 결정과 이유](docs/decisions.md)
- [API 명세](docs/api.md)
- [디자인 시스템](docs/DESIGN.md)
- [일정](docs/roadmap.md)

## 오픈소스·에셋 출처
준비 중
