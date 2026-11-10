# FLUTCH — 심장이 뛰는 순간, 플러치.

FLUTCH(플러치)는 게임과 버추얼 콘텐츠를 중심으로 스트리머와 시청자가 함께 즐기는 라이브 스트리밍 플랫폼입니다.
FLUTTER의 두근거림과 CLUTCH의 짜릿함을 결합한 이름처럼, 라이브 방송에서 생기는 모든 순간의 감정과 에너지를 중요하게 생각합니다.

**Feel the Moment. Live on FLUTCH.**

## 빠른 확인 (5분)
1. https://flutch.kro.kr 접속 → 로그인 화면의 **"테스트 계정으로 바로 로그인"**에서 **스트리머1** 선택.
2. **방송하기** → 방송 등록 → 송출 화면에서 화면 공유(또는 카메라) 미리보기 → **방송 시작**.
3. 다른 기기(휴대폰)나 다른 브라우저에서 같은 주소를 열어 방송을 시청(로그인 불필요) → 스트리머 화면에서 **방송 종료**.

## 서비스 주소
- https://flutch.kro.kr

## 테스트 계정
로그인 화면(`/login`)의 **"테스트 계정으로 바로 로그인"** 버튼 3개를 누르면 비밀번호 없이 들어갑니다.

| 버튼 | 아이디 | 용도 |
|---|---|---|
| 스트리머1 | `streamer1` | 방송 등록·송출 |
| 스트리머2 | `streamer2` | "다른 사용자" 권한 확인용 |
| 시청자 | `viewer` | 시청·채팅 |

- 아이디·비밀번호로 직접 로그인할 때의 비밀번호: **(제출 시 기재)**
- 회원가입(`/signup`)도 가능합니다(이메일·휴대폰 인증 없음).

## 실행 방법

### 실행 환경
| 항목 | 버전·값 |
|---|---|
| OS | Linux (개발은 Windows의 WSL2 Ubuntu 24.04), 운영은 Ubuntu arm64 |
| Rust | stable (백엔드) |
| Node | 22 (프론트) |
| Docker / Compose | 로컬 Postgres 16·LiveKit, 운영 배포 |
| 브라우저 | 아래 "지원 환경" 표 |

### 환경변수 예시
`.env.example`을 복사해 `.env`로 씁니다(`.env`는 커밋 금지). 비밀값은 저장소에 없습니다.

| 이름 | 예시 | 설명 |
|---|---|---|
| `BACKEND_PORT` / `VITE_PORT` / `POSTGRES_PORT` | `8080` / `5173` / `5432` | 포트 |
| `DATABASE_URL` | `postgres://flutch:flutch@localhost:5432/flutch_dev` | 로컬 DB(compose.dev 기본 계정) |
| `ALLOWED_ORIGINS` | `http://localhost:5173` | 변경 요청·WebSocket Origin 허용 목록. 운영은 `https://flutch.kro.kr` |
| `SEED_PASSWORD` | (운영 서버에서만 설정) | 시드 계정 비밀번호. 비우면 "테스트 계정으로 바로 로그인"으로만 진입 |
| `LIVEKIT_URL` / `LIVEKIT_API_KEY` / `LIVEKIT_API_SECRET` | `ws://localhost:7880` / `devkey` / `secret` | 로컬 LiveKit(개발용 고정값). 운영은 LiveKit Cloud 값 |
| `LIVE_GRACE_SECS` / `LIVE_MAX_DURATION_SECS` | (비움) | 재연결 유예 60초 / 최대 3시간 기본값. 수동 확인 때만 줄임 |
| `RUST_LOG` | `info` | 로그 수준 |

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
송출·시청은 브라우저에서 사람이 확인합니다(자동 E2E 없음). 아래 번호 절차를 순서대로 따라 하면 됩니다. 백엔드 규칙은 시나리오 테스트가 따로 검증하며, 대응은 "시나리오 ↔ 절차 대응표"에 있습니다.

### 지원 환경
송출 화면에 들어가면 브라우저가 지원 여부를 검사해 안내합니다. HTTPS(또는 localhost)가 아니면 화면 공유·카메라를 쓸 수 없습니다.

| 기능 | 지원 환경 |
|---|---|
| 화면 공유 + 시스템 소리 | Windows의 Chrome · Edge (전체 화면 공유 선택 시) |
| 화면 공유 + 탭 소리 | 데스크톱 Chrome · Edge (탭 공유 선택 시) |
| 화면 공유 (소리 없음) | 그 외 데스크톱 브라우저(예: Firefox, 맥) |
| 모바일 카메라·마이크 송출 | Android Chrome, iOS Safari (HTTPS 필수) |
| 시청 | 최신 Chrome · Edge · Safari · Firefox, 데스크톱·모바일 |

미지원 조합은 화면에서 안내합니다. 시스템 소리를 못 쓰면 "이 환경에서는 탭 소리만 송출할 수 있어요", 화면 공유를 못 쓰면 카메라 모드로 자동 전환(모바일은 화면 공유 미지원), 권한 거부 시 허용 방법을 보여 줍니다.

### 준비
- 기기 2대 권장: **A**(PC, 송출) / **B**(휴대폰 또는 다른 브라우저, 시청).

### 절차
**P1. 로그인**
1. A에서 `/login` → "테스트 계정으로 바로 로그인" → **스트리머1**.
2. 기대: 헤더에 스트리머1 닉네임이 보이고, 새로고침해도 로그인이 유지된다.

**P2. 방송 등록·수정·삭제**
1. **방송하기**에서 방송 등록(제목 2~50자, 카테고리, 태그).
2. 내 방송 관리(`/manage`)에서 제목을 수정한다.
3. 쓰지 않을 방송 하나는 삭제한다(확인 창에서 삭제).
4. 기대: 등록한 방송이 READY(대기)로 내 방송 관리에만 보이고, 수정이 반영되고, 삭제하면 사라진다.

**P3. PC 송출 (화면 공유 + 소리 + 마이크)**
1. A의 송출 화면(`/studio`)에서 방송을 고르고 "화면 선택하고 미리보기".
2. 공유 창에서 탭 또는 전체 화면을 고르고 소리 공유를 켠다(탭 소리 공유 / 시스템 소리 공유).
3. 마이크를 허용하고 미리보기·레벨 미터를 확인한 뒤 **방송 시작**.
4. 기대: 방송이 LIVE가 되고 홈 목록에 나타난다.

**P4. 다른 기기 시청**
1. B에서 같은 주소의 홈(비로그인 가능)에서 방송 카드를 눌러 시청 화면으로 들어간다.
2. 기대: 영상이 음소거로 자동 재생되고, "소리 켜기"를 누르면 A의 공유 소리·마이크가 들린다. 시청자 수가 오르고 경과 시간이 표시된다.

**P5. 방송 종료**
1. A에서 **방송 종료** → 확인.
2. 기대: A·B 모두 "방송이 종료되었어요"(최고 시청자 수·방송 시간)가 보이고 홈 목록에서 사라진다. 종료된 방송은 다시 시작할 수 없다.

**P6. 모바일 송출 (카메라·마이크)**
1. 휴대폰(HTTPS 주소)에서 스트리머1로 로그인 → 방송 등록 → 송출 화면 → "카메라 켜고 미리보기" → 권한 허용 → **방송 시작**.
2. "후면 카메라로" / "전면 카메라로"로 전환한다.
3. PC에서 시청하고, 휴대폰에서 방송 종료.
4. 기대: 화면 공유 없이 카메라 모드로 열리고, 전환하면 영상이 바뀌며, 시청·종료는 P4·P5와 같다.

**P7. 방송 상태 구분**
1. 방송 등록 직후 시청 주소(`/live/<방송 ID>`)를 B에서 연다 → "곧 방송이 시작돼요" 대기 화면.
2. 방송을 시작하면 B가 자동으로 재생된다(LIVE).
3. 종료하면 "방송이 종료되었어요"(ENDED).
4. 기대: 대기·방송 중·재연결·종료 네 상태가 화면에서 구분된다(재연결은 P9).

**P8. 방송 중 정보 변경과 삭제**
1. LIVE 중에 방송 정보에서 제목·카테고리를 바꾼다.
2. 기대: B의 시청 화면·홈 목록에 새로고침 없이 반영된다. 녹화 여부는 바꿀 수 없다.
3. LIVE 방송을 삭제하면 송출이 먼저 끊기고(종료) 삭제된다.

**P9. 송출 끊김과 복구**
1. 송출 중 A의 탭을 닫는다(또는 네트워크를 끊는다).
2. 기대: B에 "재연결 대기"가 뜬다. 60초 안에 A가 돌아와 "방송을 이어서 할까요?"에서 이어 가면 같은 방송이 계속되고, 60초가 지나면 자동으로 ENDED가 된다.
3. 화면 공유만 중지하면 방송은 유지되고 B에 "잠시 화면 공유가 중단됐어요", A에 "다시 공유하기"가 보인다.
4. 같은 방송을 다른 기기에서 시작하면 "이 기기로 전환" 확인 후 새 기기로 넘어가고, 기존 기기의 송출은 끊긴다.

**P10. 재시작 후 조회**
1. 서버 재시작 뒤(배포 직후 포함) 홈·방송 상세·내 방송 관리를 다시 연다.
2. 기대: 등록한 방송 정보가 그대로 조회된다. LIVE였던 방송은 시작 시 LiveKit 실제 방 상태와 동기화된다.

**P11. 인증·인가**
1. 비로그인(시크릿 창): 홈 목록·시청 화면이 열린다. 송출·방송 관리는 로그인을 요구한다.
2. 소유자(스트리머1): 자기 방송의 수정·삭제·송출 시작이 된다.
3. 다른 사용자(스트리머2): 스트리머1 방송의 수정·삭제·송출 시작이 거절된다(403). 시청은 가능하다.
4. 같은 아이디로 비밀번호를 5번 연속 틀리면 5분간 잠긴다. 로그아웃하면 이후 요청은 401.
5. 기대: 허용·거절이 서버 응답으로 구분된다(화면에서 숨기는 것만으로 처리하지 않음).

**P12. 가입·계정**
1. `/signup`에서 아이디·비밀번호·닉네임·약관 동의로 가입 → 로그아웃 → 다시 로그인.
2. 닉네임은 변경 후 30일간 다시 바꿀 수 없고, 비밀번호를 바꾸면 다른 기기의 로그인이 풀린다.

### 시나리오 ↔ 절차 대응표
시나리오 테스트는 `backend/tests/scenarios/`에 있고 `cd backend && cargo test`로 실행합니다(실제 Postgres, LiveKit은 가짜).

| 시나리오 테스트 | 검증 내용 | 절차 |
|---|---|---|
| `auth_signup_login_logout` | 가입 → 로그인 → 내 정보 → 로그아웃 후 401 | P1, P11, P12 |
| `auth_signup_validation_and_duplicates` | 가입 입력 검증·중복 | P12 |
| `auth_test_login_seed_accounts` | 시드 3계정 테스트 로그인 | P1 |
| `auth_lockout` | 5회 실패 → 잠금 | P11 |
| `auth_rate_limits_by_ip` | IP별 요청 제한 | (서버 내부) |
| `auth_requires_login_and_acts_only_on_own_account` | 로그인 필수·본인 계정만 | P11 |
| `auth_password_change_revokes_other_sessions` | 비밀번호 변경 시 다른 세션 만료 | P12 |
| `auth_nickname_change_30day_limit` | 닉네임 30일 제한 | P12 |
| `broadcast_owner_crud` | 등록 → 수정 → 삭제 | P2 |
| `broadcast_other_user_forbidden` | 다른 사용자 403, 비로그인 401 | P11 |
| `broadcast_public_read` | 비로그인 목록·상세 200 | P4, P11 |
| `broadcast_state_transitions` | 허용 전이만, LIVE·ENDED 수정 범위 | P5, P7, P8 |
| `broadcast_limits` | 동시 LIVE 1개, READY 5개 | P2 |
| `broadcast_delete_live_ends_first` | LIVE 삭제 시 종료 후 삭제 | P8 |
| `broadcast_persists_after_restart` | 재시작 후 조회 | P10 |
| `live_start_owner_only` | 소유자만 publish 토큰 | P3, P11 |
| `live_viewer_token_rules` | 시청 토큰은 누구나, subscribe 전용 | P4, P11 |
| `live_end_closes_room` | 종료 시 ENDED + 방 닫기 | P5 |
| `live_disconnect_grace_auto_end` | 퇴장 → 60초 유예 → 자동 종료 | P9 |
| `live_takeover_other_device` | 다른 기기로 전환 | P9 |
| `live_max_duration_auto_end` | 3시간 자동 종료 | (수동 절차 없음) |
| `live_webhook_signature_invalid_rejected` | 웹훅 서명 검증 | (서버 내부) |
| `live_viewer_count` | 시청자 수 집계 | P4 |
| `live_restart_sync` | 시작 시 LiveKit 상태 동기화 | P10 |
| `ws_receives_broadcast_updated_and_ended` | 변경·종료 실시간 반영 | P5, P8 |
| `ws_split_event_hides_anonymous_from_public` | 연결별 이벤트 필터링 | (후원 절차 추가 시) |
| `ws_unknown_or_hidden_broadcast_404` | 없는·숨긴 방송 구독 404 | (서버 내부) |
| `ws_origin_rejected` | WebSocket Origin 검사 | (서버 내부) |
| `ws_lagged_sends_resync` | 지연 연결에 재동기화 | (서버 내부) |
| `mutating_request_without_or_with_foreign_origin_is_forbidden`, `mutating_request_with_allowed_origin_passes`, `read_request_needs_no_origin`, `webhooks_skip_origin_check` | CSRF Origin 검사 | (서버 내부) |
| `schema_one_live_per_user` | DB 제약: 사용자당 LIVE 1개 | P2 |
| `schema_ledger_is_append_only`, `schema_payment_key_unique`, `schema_rejects_negative_balance` | DB 제약: 원장·결제·잔액 | (후원 절차 추가 시) |
| `health_check_reports_ok_with_request_id`, `health_check_reports_503_when_db_is_down` | 헬스체크 | 배포 후 `GET /api/v1/health` |
| `unknown_api_path_returns_not_found_error_format` | 에러 응답 형식 | (서버 내부) |

사람이 보는 절차 중 시나리오가 없는 것: P3의 화면 공유·소리, P6의 모바일 카메라, P4의 영상·소리 재생(브라우저·기기 의존).

## 알려진 한계
> 제출일에 `/ponytail-debt` 결과로 갱신합니다.
- 토스페이먼츠 **테스트 모드**만 지원합니다. 충전한 비트의 환불은 지원하지 않습니다.
- 이메일·휴대폰 본인 확인이 없어 탈퇴 후 새 아이디 재가입이 가능합니다(동일인 판별 불가).
- 화면 공유 소리는 브라우저 기능에 따라 다릅니다(위 지원 환경 표). 화면 공유는 모바일에서 지원하지 않습니다.

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
