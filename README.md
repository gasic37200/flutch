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

## 배포
main에 머지되면 `.github/workflows/deploy.yml`이 arm64 이미지(`ghcr.io/gasic37200/flutch-backend`, `flutch-web`)를 GitHub의 `ubuntu-24.04-arm` 러너에서 빌드해 GHCR에 올리고, SSH로 서버(`~/flutch/`)에서 `docker compose pull && up -d` 한 뒤 `https://flutch.kro.kr/api/v1/health`를 확인합니다. 서버에서는 Rust를 빌드하지 않습니다. `flutch-web` 이미지는 프론트 정적 파일과 nginx 설정을 함께 담습니다.

**GitHub Secrets** (저장소 Settings → Secrets and variables → Actions)
| 이름 | 값 |
|---|---|
| `DEPLOY_HOST` | 서버 IP 또는 도메인 |
| `DEPLOY_USER` | 서버 SSH 사용자 (`ubuntu`) |
| `DEPLOY_SSH_KEY` | 서버 접속용 개인키 전체(`-----BEGIN ...` 포함) |
| `GHCR_READ_TOKEN` | `read:packages` 권한 classic PAT. 서버가 비공개 이미지를 pull 할 때 쓴다 |

**서버 준비 (한 번)**
1. Docker·Compose 설치, 80/443 개방(서버 iptables + OCI Security List). 도메인 A 레코드가 서버 IP를 가리켜야 한다.
2. `~/flutch/.env` 작성(`.env.example`의 "운영 서버 전용" 항목, 파일 권한 600). `POSTGRES_PASSWORD`는 `openssl rand -hex 24`.
3. `deploy/compose.prod.yml`, `backup.sh`, `init-cert.sh`를 `~/flutch/`에 둔다(첫 배포 workflow가 올려 주므로 이후엔 자동).
4. 최초 인증서: `cd ~/flutch && ./init-cert.sh` (이메일 없이 발급, 80번 포트가 비어 있을 때). 갱신은 `certbot` 컨테이너가 12시간마다 확인하고 `web`이 12시간마다 nginx를 reload 한다.
5. DB 백업 cron: `crontab -e` → `0 4 * * * /home/ubuntu/flutch/backup.sh >> /home/ubuntu/flutch/backups/backup.log 2>&1` (서버 시간대가 KST). 요일별 파일 7개가 돌아가며 덮어써져 7일 보관이 된다. Oracle Object Storage 업로드는 `.env`에 `BACKUP_S3_*`를 채우면 켜진다.

**수동 재배포** (Actions → Deploy → Run workflow, 또는 서버에서)
```bash
cd ~/flutch && docker compose -f compose.prod.yml pull && docker compose -f compose.prod.yml up -d
# 특정 커밋으로 되돌리기: FLUTCH_TAG=<커밋 SHA> docker compose -f compose.prod.yml up -d
```

## 기능 확인
준비 중

## 문서
- [공모전 요구사항](docs/spec.md)
- [설계 결정과 이유](docs/decisions.md)
- [API 명세](docs/api.md)
- [디자인 시스템](docs/DESIGN.md)
- [일정](docs/roadmap.md)

## 오픈소스·에셋 출처
| 항목 | 출처·라이선스 |
|---|---|
| Pretendard (본문 글꼴) | [orioncactus/pretendard](https://github.com/orioncactus/pretendard), SIL OFL 1.1, jsDelivr CDN |
| Chakra Petch (영문 제목 글꼴) | [Google Fonts](https://fonts.google.com/specimen/Chakra+Petch), SIL OFL 1.1 |
| JetBrains Mono (숫자 글꼴) | [Google Fonts](https://fonts.google.com/specimen/JetBrains+Mono), SIL OFL 1.1 |
| 욕설 목록 (`backend/data/profanity_ko.txt`) | [LDNOOBW List of Dirty, Naughty, Obscene, and Otherwise Bad Words](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) `ko`, CC BY 4.0 — 오탐 항목 제거·일부 추가 |
| 흔한 비밀번호 상위 100 (`backend/data/common_passwords.txt`) | [SecLists](https://github.com/danielmiessler/SecLists) `xato-net-10-million-passwords-100.txt`, MIT |
| 로고·마스코트·배너·배경 텍스처 (`frontend/public/brand/`) | **AI 생성 이미지**를 가공해 사용 (원본·가공본은 `docs/design-refs/`) |
| livekit-client (WebRTC 송출·시청 SDK) | [livekit/client-sdk-js](https://github.com/livekit/client-sdk-js), Apache-2.0, npm |
