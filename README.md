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
| `KAKAO_REST_KEY` / `KAKAO_REDIRECT_URI` | (비움) | 카카오 로그인. 둘 다 있어야 켜지고, 없으면 로그인 화면에서 카카오 버튼이 숨겨짐. 설정은 아래 "카카오 로그인 설정" |
| `KAKAO_CLIENT_SECRET` / `KAKAO_ADMIN_KEY` | (비움) | 콘솔에서 Client Secret을 켠 경우만 / 탈퇴 시 카카오 연결 끊기용(없으면 그 단계만 건너뜀) |
| `RUST_LOG` | `info` | 로그 수준 |
| `GRAFANA_ADMIN_PASSWORD` | (운영 서버에서만 설정) | Grafana 관리자 비밀번호. 로컬 개발에는 필요 없음 |

### 카카오 로그인 설정
키가 없어도 서비스는 정상이고 아이디·테스트 계정으로만 로그인됩니다. 켜려면 카카오 개발자 콘솔(developers.kakao.com)에서:
1. 애플리케이션을 만들고 **앱 키**의 REST API 키를 `KAKAO_REST_KEY`로, Admin 키를 `KAKAO_ADMIN_KEY`로 둡니다.
2. **앱 > 플랫폼 > Web**에 사이트 도메인(`https://flutch.kro.kr`, 로컬은 `http://localhost:5173`)을 등록합니다.
3. **제품 설정 > 카카오 로그인**을 켜고 **Redirect URI**에 `https://flutch.kro.kr/api/v1/auth/kakao/callback`(로컬은 `http://localhost:5173/api/v1/auth/kakao/callback`)을 등록합니다. `KAKAO_REDIRECT_URI`에는 똑같은 값을 넣습니다.
4. **동의항목**에서 `닉네임(profile_nickname)`만 켭니다(이메일은 요청하지 않습니다).
5. **보안 > Client Secret**을 만들고 활성화했다면 `KAKAO_CLIENT_SECRET`에 넣습니다.
6. 운영 서버 `~/flutch/.env`에 위 값을 넣고 백엔드를 다시 띄웁니다.

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
4. 최초 인증서: `cd ~/flutch && ./init-cert.sh` (이메일 없이 발급, 80번 포트가 비어 있을 때). kro.kr은 Let's Encrypt 공유 한도로 거절이 잦아, 서버 `.env`에 `ZEROSSL_EAB_KID`·`ZEROSSL_EAB_HMAC_KEY`가 있으면 ZeroSSL로 발급한다. 갱신은 `certbot` 컨테이너가 12시간마다 확인하고 `web`이 12시간마다 nginx를 reload 한다.
5. 모니터링용 `.env` 값 `GRAFANA_ADMIN_PASSWORD`(필수, `openssl rand -hex 16`). 없으면 `docker compose`가 실패한다.
6. DB 백업 cron: `crontab -e` → `0 4 * * * /home/ubuntu/flutch/backup.sh >> /home/ubuntu/flutch/backups/backup.log 2>&1` (서버 시간대가 KST). 요일별 파일 7개가 돌아가며 덮어써져 7일 보관이 된다. Oracle Object Storage 업로드는 `.env`에 `BACKUP_S3_*`를 채우면 켜진다.

**모니터링 (운영)**
- 스택: Prometheus(15일)·Loki(7일)·Alloy(컨테이너 로그 수집)·node-exporter·Grafana. 모두 `compose.prod.yml`에 있고 외부 포트는 없다(`web`의 nginx가 `/grafana/`만 연다). 설정·대시보드는 `deploy/monitoring/`(workflow가 서버로 올린다). 스택 전체 메모리 상한은 약 1.7GB.
- Grafana: `https://flutch.kro.kr/grafana/` → 사용자 `admin` / 서버 `.env`의 `GRAFANA_ADMIN_PASSWORD`로 로그인(익명 접근 꺼짐). 대시보드는 "서비스 개요"(요청량·에러율·p95·LIVE 수·WS 연결 수·후원·CPU/메모리/디스크)와 "로그 검색"(서비스·레벨 필터, 검색어에 에러 화면의 `request_id`를 넣으면 해당 요청 로그가 나온다).
- `/metrics`는 백엔드가 Prometheus 형식으로 내보내며 nginx가 외부 요청을 404로 막는다. Prometheus가 도커 네트워크로 직접 수집한다. 알림은 없다.
- 로그에는 비밀번호·세션 쿠키·LiveKit 토큰·토스 시크릿·채팅 본문을 남기지 않는다(요청 로그는 request_id·method·route·status·latency·user_id만).

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
3. 직접 가입한 계정으로 `/me` 아래 "회원 탈퇴" → 비밀번호 입력(잔액이 있으면 "비트 소멸과 위 내용을 이해했어요" 체크) → "탈퇴" → 로그아웃 상태로 홈. 같은 아이디로 로그인·재가입이 안 된다. 방송 중에는 "방송을 먼저 종료해 주세요", 테스트 계정은 "테스트 계정은 탈퇴할 수 없어요"가 뜬다.

4. (카카오 키를 설정한 환경) `/login`의 "카카오로 로그인" → 카카오 인증 → 닉네임 설정 화면(카카오 닉네임이 채워져 있음) → 약관 동의 후 가입하면 로그인됩니다. 로그아웃 후 같은 방법으로 다시 로그인하면 닉네임 화면 없이 같은 계정으로 들어갑니다. `/me`에는 비밀번호 변경이 없고, "회원 탈퇴"는 비밀번호 없이 되며 카카오 계정 설정의 "연결된 서비스"에서 FLUTCH가 사라집니다.

**P13. 채팅**
1. 방송 중인 방송을 B(스트리머2)로 열고 오른쪽 채팅 창(모바일은 영상 아래 "채팅" 탭)에서 메시지를 입력해 Enter → A 화면에도 바로 보인다. 비로그인(시크릿 창)은 읽기만 되고 입력창 자리에 "로그인하고 채팅하기" 버튼이 나온다. 방송 전(READY)·종료 후에는 입력창 대신 "방송 중에만 채팅할 수 있어요"가 보이고 기록은 계속 읽힌다.
2. 1초 안에 연달아 보내면 입력창 아래에 서버 문구가 뜨며 거절되고, 같은 말을 30초 안에 3번 보내면 "같은 메시지를 반복할 수 없어요"가 뜬다. 200자를 넘게 입력할 수 없고 글자 수가 오른쪽 아래에 보인다.
3. 욕설(예: 일반 비속어)은 `***`로 가려 보이고, FLUTCH 금지어는 전송되지 않는다.
4. 내 메시지 오른쪽 "⋯" 메뉴(모바일은 항상 보임)에서 5분 안에 "수정"하면 "(수정됨)"이 붙고, "삭제"하면 확인 창 뒤에 "삭제된 메시지입니다"가 된다. 방송 소유자(A)는 남의 메시지에도 "삭제"가 보이고 지울 수 있다.
5. 목록 맨 위로 스크롤하면 이전 기록이 이어서 불러와지고, 위를 보는 중에 새 메시지가 오면 "새 메시지 n개" 버튼이 나온다. PC 스튜디오의 방송 중 오른쪽 창과 모바일 송출 화면(최근 5개 반투명)에도 같은 채팅이 보인다.
6. 차단: 방송 소유자(A)가 채팅 닉네임을 누르면 창이 뜨고 임시차단(1분·10분·1시간)·영구 밴(확인 창)·차단 해제·"이 시청자의 채팅 기록"이 보인다(시청자 화면의 창에는 관리 메뉴가 없다). 차단하면 채팅창에 "OOO님의 채팅이 10분간 제한되었어요"/"OOO님이 차단되었어요"가 뜨고, 차단된 사용자의 입력창은 "n분 후 채팅할 수 있어요"(임시, 남은 시간이 줄어듦) 또는 "이 채널에서 채팅이 제한되었어요"(영구)로 바뀐다. 영구 밴이면 그 사람의 메시지가 모두 "삭제된 메시지입니다"가 된다. 해제하면 차단된 쪽 입력창이 10초 안에 다시 열린다. 내 스트리머 페이지 "차단 목록" 탭에서 남은 시간을 보고 해제할 수 있다.

**P14. 다시보기**
1. 녹화를 켠 방송을 송출하고 종료한다(배포 환경에서만 영상이 만들어진다). 종료 화면에 "다시보기 준비 중"이 보이다가 준비되면 "다시보기 보기"로 바뀐다.
2. 사이드바 "다시보기"(`/replay`)에서 최신순 목록과 카테고리 필터를 확인하고, 카드를 눌러 상세에서 재생한다. 탐색·재생 바, 배속(0.5~2배), 볼륨, 전체화면이 되고 오른쪽에 채팅 기록(읽기 전용)이 보인다.
3. 같은 브라우저로 두 번 재생해도 조회수는 하루(KST) 한 번만 오른다. 로그인 여부와 상관없이 누구나 볼 수 있다.
4. 스트리머 페이지(`/users/<id>`)의 "방송" 탭에서 지난 방송, "다시보기" 탭에서 다시보기를 본다. 본인에게만 "내 방송 관리" 버튼이 보인다. 홈의 "최근 다시보기"와 탐색 검색의 다시보기 결과도 같은 목록이다.
5. 기대: 처리 중·녹화 실패·보관 기간 지남(30일 뒤) 상태가 각각 안내 문구로 구분된다.

**P15. 화면 + 웹캠 (PIP)**
1. A의 송출 화면에서 화면 공유로 방송을 시작한 뒤 "웹캠 같이 보내기 (PIP)"를 켠다. 웹캠 위치(네 모서리)를 고르면 미리보기에도 같은 자리에 겹쳐 보인다.
2. B의 시청 화면에서 화면 위 선택한 모서리에 웹캠이 작게 보이는지 확인한다. 위치를 바꾸면 새로고침 없이 따라 옮겨지고, 웹캠을 끄고 켜면 사라졌다 다시 나타난다.
3. B에서 컨트롤 바의 카메라 버튼(또는 웹캠 화면 탭)으로 웹캠을 숨기고, 카메라 버튼으로 다시 보인다.
4. A에서 "웹캠만"으로 바꾸거나 화면 공유가 끊기면 B에서 웹캠이 PIP 대신 화면 전체로 커진다.
5. 기대: 화면과 웹캠이 별도 트랙으로 전송되어 A가 다른 창(게임)으로 전환해도 두 영상이 끊기지 않는다.

**P17. 지갑·충전·후원**
1. 로그인(시청자)한 뒤 헤더 비트 알약 또는 "내 지갑"(`/wallet`)에서 잔액과 내역을 본다. 유형(충전·후원·클러치·SUPER BURST·소멸)과 월 필터로 좁혀지고, 유료 클러치는 방송별로 묶여 한 줄로 나온다. "충전하기" → 금액(1,000/5,000/10,000/50,000 또는 직접 입력 1,000~1,000,000) → "결제하기"로 토스 결제위젯이 열린다. 결제를 마치면 `/payments/success`가 승인하고 "충전했어요!"와 잔액이 보인다. 결제창을 닫거나 실패하면 `/payments/fail`이 "비트는 빠져나가지 않았어요"를 보여 준다. 보유 한도(3,000,000)를 넘는 금액은 막힌다.
2. 방송 중인 방송을 시청자로 열고 "후원하기": 프리셋(1,000/5,000/10,000) 또는 직접 입력, 메시지(100자), "익명으로 후원" 체크. 잔액이 충분하면 바로 후원되고 잔액 알약이 응답 값으로 줄어든다.
3. 잔액이 모자라면 버튼이 "N비트 충전하고 후원하기"로 바뀌고, 누르면 결제위젯 → 승인 뒤 "후원했어요!"와 "방송으로 돌아가기"가 나온다. 결제 사이 방송이 끝났거나 차단됐으면 "후원 없이 충전만" 안내가 나온다.
4. 후원이 들어오면 시청 화면·스트리머 화면 영상 위에 알림이 10초(여러 건은 순서대로), 10,000비트 이상은 노란 큰 카드로 뜨고 채팅창에 강조 한 줄이 남는다. 익명 후원은 시청자 화면에 "익명의 후원자", 스트리머 화면에는 실명 + "익명" 배지가 보이며, 스트리머가 "넘기기"를 누르면 모든 시청자 화면의 현재 알림이 넘어간다.
5. 시청 화면 영상 아래 "이 방송 후원"(모바일은 "후원" 탭)에 합계(유료 클러치·버스트 포함)와 최근 후원 목록이 보인다. 스트리머 페이지 "후원 내역" 탭(본인만)에서 방송을 골라 익명 후원도 실명으로 보고 "차단"으로 바로 차단할 수 있고, "통계" 탭에서 누적·최근 7일·방송별 받은 비트를 본다.

**P16. 팔로우·PULSE**
1. B에서 `/login` → **시청자**로 로그인 → 스트리머1의 스트리머 페이지(방송 카드나 이름을 눌러 이동) → **팔로우**.
2. 기대: 버튼이 "팔로잉"으로 바뀌고 팔로워 수가 1 늘어난다. 다시 누르면 취소된다. 본인 페이지에는 팔로우 버튼이 없다.
3. A에서 스트리머1이 방송을 시작하면 B의 사이드바 "PULSE"(팔로우한 채널 중 라이브)와 `/pulse`의 "지금 라이브"에 나타나고, 방송 중이 아니면 "오프라인"에 마지막 방송 시각과 함께 나온다. 비로그인은 사이드바가 "인기 라이브"이고 `/pulse`가 로그인을 안내한다.

**P18. SUPER CLUTCH 회전·홈 히어로**
1. 스트리머1·스트리머2로 각각 방송을 시작하고 시청자로 홈을 연다. 진행 중인 SUPER CLUTCH가 없으면 히어로는 게이지가 가장 높은 방송(모두 0%면 시청자 수 1위)이고 오른쪽 CLUTCH TIME 패널에 그 방송 게이지와 "지금, 클러치할 시간!"(80% 이상은 "곧 터져요!", 발동 후 쿨다운은 "다음 클러치까지 n분")이 보인다. 모바일은 영상 카드 아래 한 줄 패널.
2. 두 방송에서 각각 SUPER BURST(1,000비트) 또는 클러치로 게이지를 100%로 만든다. 10초 안에 히어로가 SUPER CLUTCH 배지·그라데이션 테두리로 바뀌고, 패널에 "다음 방송까지" 카운트다운·30초 진행 바·"지금 보러 가기"·진행 중 목록(남은 시간)이 보인다. 시청 화면 영상 왼쪽 위에도 SUPER CLUTCH 배지가 붙는다.
3. 기대: 30초마다 히어로가 두 방송을 번갈아 보여 주고, 새로 터진 방송은 바로 다음 차례에 나온다. 목록의 방송을 누르면 바로 시청 화면으로 간다. 발동 5분 뒤나 방송을 끝내면 회전에서 빠지고(노출 중이면 바로 다음 방송), 서버를 다시 띄워도 같은 순서·남은 시간이 이어진다.

### 시나리오 ↔ 절차 대응표
시나리오 테스트는 `backend/tests/scenarios/`에 있고 `cd backend && cargo test`로 실행합니다(실제 Postgres, LiveKit·토스는 가짜).

| 시나리오 테스트 | 검증 내용 | 절차 |
|---|---|---|
| `auth_signup_login_logout` | 가입 → 로그인 → 내 정보 → 로그아웃 후 401 | P1, P11, P12 |
| `auth_signup_validation_and_duplicates` | 가입 입력 검증·중복 | P12 |
| `auth_test_login_seed_accounts` | 시드 3계정 테스트 로그인 | P1 |
| `auth_lockout` | 5회 실패 → 잠금 | P11 |
| `follow_toggle_and_count` | 팔로우·중복 팔로우·언팔로우·탈퇴한 팔로워 제외 | P16 |
| `follow_self_rejected` | 자기 자신 팔로우 거절 | P16 |
| `pulse_live_first_limit_and_withdrawn` | PULSE 라이브 먼저·limit·비로그인 401·탈퇴 스트리머 제외 | P16 |
| `auth_kakao_new_user_signup`, `auth_kakao_existing_user_login` | 카카오 첫 로그인 → 닉네임 설정 → 가입(지갑 생성), 재로그인은 같은 계정 | P12 |
| `auth_kakao_state_mismatch_rejected`, `auth_kakao_signup_rules_and_forged_cookie` | state 불일치·동의 취소·위조 임시 쿠키 거절, 닉네임 규칙 | (서버 내부) |
| `auth_kakao_withdraw_unlinks`, `auth_password_account_withdraw_needs_password` | 카카오 계정은 비밀번호 없이 탈퇴 + 연결 끊기, 같은 카카오 계정 재가입은 새 계정 | P12 |
| `auth_rate_limits_by_ip` | IP별 요청 제한 | (서버 내부) |
| `auth_requires_login_and_acts_only_on_own_account` | 로그인 필수·본인 계정만 | P11 |
| `auth_password_change_revokes_other_sessions` | 비밀번호 변경 시 다른 세션 만료 | P12 |
| `auth_nickname_change_30day_limit` | 닉네임 30일 제한 | P12 |
| `auth_withdraw_forfeits_balance`, `auth_withdraw_rules` | 탈퇴: 잔액 소멸 원장·세션 전부 만료·재로그인·재가입 불가·닉네임 30일 보류, LIVE 중·확인 없음·테스트 계정 거절 | P12 |
| `broadcast_owner_crud` | 등록 → 수정 → 삭제 | P2 |
| `chat_live_only`, `chat_guest_read_only` | LIVE 중에만 작성, 비로그인은 읽기만 | P13 |
| `chat_edit_author_only_within_5min`, `chat_delete_author_or_owner` | 수정(작성자·5분)·삭제(작성자·방송 소유자) | P13 |
| `chat_rate_limit_and_repeat`, `chat_profanity_masked_banned_word_blocked` | 도배 제한, 욕설 가림·금지어 차단 | P13 |
| `chat_banned_user_cannot_write` | 임시·영구 차단 사용자 작성·수정 거절 | P13 |
| `ban_blocks_chat_across_broadcasts`, `timeout_expires_automatically` | 차단은 그 스트리머의 모든 방송에 적용, 임시차단은 시간이 지나면 자동 해제 | P13 |
| `ban_effects_matrix`, `ban_only_by_owner` | 차단 효과 표(작성·수정 불가, 시청 가능), 본인만 차단 가능 | P13 |
| `permanent_ban_hides_current_messages` | 영구 밴 시 현재 방송 글 삭제 표시 + 시스템 알림 | P13 |
| `ban_history_unban_and_validation`, `viewer_chat_history` | 이력 추가만·해제·자기 자신 거절, 시청자 채팅 기록 | P13 |
| `chat_badges_streamer_and_donor`, `chat_events_published` | 배지, 실시간 이벤트 | P13 |
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
| `recording_saved_after_end_survives_restart` | 녹화 완료 통보 후 재시작해도 다시보기 조회 | P5, P10 |
| `recording_failure_does_not_break_broadcast` | 녹화 시작·진행 실패에도 방송은 정상 종료 | P5 |
| `recording_deleted_with_broadcast_hide` | 방송 삭제 시 녹화 파일 삭제(실패 시 재시도) | P8 |
| `recording_retention_cleanup` | 30일·16GB 초과 정리, 종료 통보 없는 녹화 실패 처리 | (서버 내부) |
| `recording_off_without_record_flag` | 녹화 끄면 Egress 시작 안 함 | P2 |
| `recording_views_once_per_day` | 다시보기 조회수 시청자당 하루 1회 | (서버 내부) |
| `live_takeover_other_device` | 다른 기기로 전환 | P9 |
| `live_max_duration_auto_end` | 3시간 자동 종료 | (수동 절차 없음) |
| `live_webhook_signature_invalid_rejected` | 웹훅 서명 검증 | (서버 내부) |
| `live_viewer_count` | 시청자 수 집계 | P4 |
| `live_restart_sync` | 시작 시 LiveKit 상태 동기화 | P10 |
| `live_publisher_never_joins_auto_end` | 송출 시작 후 끝내 접속 못 하면 60초 뒤 종료 | P9 |
| `live_subscriber_same_identity_keeps_grace` | 같은 신원 시청 참가는 유예를 풀지 않음 | (서버 내부) |
| `ws_receives_broadcast_updated_and_ended` | 변경·종료 실시간 반영 | P5, P8 |
| `ws_split_event_hides_anonymous_from_public` | 연결별 이벤트 필터링 | P15 |
| `ws_unknown_or_hidden_broadcast_404` | 없는·숨긴 방송 구독 404 | (서버 내부) |
| `ws_origin_rejected` | WebSocket Origin 검사 | (서버 내부) |
| `ws_lagged_sends_resync` | 지연 연결에 재동기화 | (서버 내부) |
| `mutating_request_without_or_with_foreign_origin_is_forbidden`, `mutating_request_with_allowed_origin_passes`, `read_request_needs_no_origin`, `webhooks_skip_origin_check` | CSRF Origin 검사 | (서버 내부) |
| `schema_one_live_per_user` | DB 제약: 사용자당 LIVE 1개 | P2 |
| `schema_ledger_is_append_only`, `schema_payment_key_unique`, `schema_rejects_negative_balance` | DB 제약: 원장·결제·잔액 | P17 |
| `donation_confirm_idempotent`, `payment_concurrent_confirm_credits_once` | 같은 승인·웹훅 반복·동시 승인 → 한 번만 적립, 같은 결과 | P17 |
| `clutch_access_rules` | 비로그인 401, 내 방송 403, 비LIVE 409 | (클러치 절차 추가 시) |
| `clutch_free_combo_then_paid` | 30초 무료 5회(COMBO) → 6번째 1비트 차감·원장·후원 기록, 잔액 0이면 409 | (클러치 절차 추가 시) |
| `clutch_gauge_decay_window` | 5분 지난 점수는 게이지에서 빠짐, 80% 임박 | (클러치 절차 추가 시) |
| `super_clutch_trigger_reset_and_cooldown` | 100% 동시 요청에도 발동 1회, 게이지 0, 쿨다운 중 재발동 없음 | (클러치 절차 추가 시) |
| `clutch_banned_rules`, `clutch_super_event_published` | 영구 밴은 무료만 허용·임시차단은 둘 다, `clutch.super` 이벤트 | (클러치 절차 추가 시) |
| `clutch_gauge_event_throttled` | `clutch.gauge`는 방송당 0.5초에 1번, 사이 값은 마지막 것만 | (클러치 절차 추가 시) |
| `burst_debits_and_boosts_gauge` | SUPER BURST 1,000비트 차감·원장 burst·후원 기록, 혼자 1회로 SUPER CLUTCH 발동, 쿨다운 중엔 게이지만 상승, 잔액 부족 409 | (클러치 절차 추가 시) |
| `burst_anonymous_hidden_from_public` | 익명 버스트는 공개 이벤트·목록에서 후원자 숨김, 소유자에게만 실명 | (클러치 절차 추가 시) |
| `burst_rules`, `burst_concurrent_only_balance_succeeds` | 비로그인 401·내 방송 403·영구 밴 403·임시차단 허용·비LIVE 409, 동시 2건은 잔액만큼만 성공 | (클러치 절차 추가 시) |
| `clutch_rotation` | 진행 중 SUPER CLUTCH 30초 교대, 새 발동은 바로 다음, 진행 중 재발동 없음, 방송 종료 즉시 제외(다음이 새 슬롯), 5분 뒤 제외 | P18 |
| `hero_modes`, `rotation_survives_restart` | 히어로 empty·top_viewers·top_gauge·super_clutch, 진행 중 목록 최대 5, 30초 슬롯, 새 앱 인스턴스도 같은 슬롯 | P18 |
| `donation_amount_mismatch_rejected` | 주문과 다른 금액 승인 거절(토스 호출 없음) | P17 |
| `donation_after_end_becomes_charge`, `banned_user_cannot_donate` | 승인 시 방송 종료·영구 밴이면 충전만, 주문 단계 거절 | P17 |
| `donation_live_only`, `donation_input_rules`, `banned_user_cannot_donate` | 잔액 후원: LIVE 중에만, 자기 방송·금지어·금액·잔액 부족·영구 밴 거절, 게이지 반영 | P17 |
| `donation_balance_conditional_debit` | 동시 잔액 후원 2건 → 잔액만큼만 성공, 음수 잔액 없음 | (서버 내부) |
| `donation_anonymous_name_visibility` | 익명 후원: 공개 목록·비로그인·다른 사용자 이벤트는 익명, 소유자만 실명, 후원자 배지 | P17 |
| `donation_skip_owner_only`, `donation_created_on_charge_and_donate` | 알림 스킵은 소유자만, 충전하고 후원하기도 알림 | P17 |
| `donation_list_totals_and_pages`, `donation_stats_kst_7_days`, `donation_stats_only_mine` | 방송별 후원 내역·합계, 스트리머 통계(KST 최근 7일) | P17 |
| `charge_limits_and_order_expiry` | 충전 1,000~1,000,000·보유 3,000,000, 30분 주문 만료 | P17 |
| `payment_toss_rejected_or_unavailable`, `payment_webhook_secret_mismatch_ignored`, `payment_confirm_owner_only` | 토스 거절·통신 실패 복구, 웹훅 재확인, 주문자만 확정 | (서버 내부) |
| `health_check_reports_ok_with_request_id`, `health_check_reports_503_when_db_is_down` | 헬스체크 | 배포 후 `GET /api/v1/health` |
| `unknown_api_path_returns_not_found_error_format` | 에러 응답 형식 | (서버 내부) |

사람이 보는 절차 중 시나리오가 없는 것: P3의 화면 공유·소리, P6의 모바일 카메라, P4의 영상·소리 재생(브라우저·기기 의존).

## 알려진 한계
> 제출일에 `/ponytail-debt` 결과로 갱신합니다.
- 토스페이먼츠 **테스트 모드**만 지원합니다. 충전한 비트의 환불은 지원하지 않습니다.
- 카카오 로그인은 아이디 계정과 연결되지 않는 별도 계정입니다. 카카오 가입 대기용 서명 키는 서버 시작 때 새로 만들어, 재시작하면 가입 도중이던 사람(10분 이내)은 처음부터 다시 로그인해야 합니다.
- 이메일·휴대폰 본인 확인이 없어 탈퇴 후 새 아이디 재가입이 가능합니다(동일인 판별 불가).
- 화면+웹캠(PIP) 방송의 녹화는 LiveKit 기본 배치(화면 크게 + 웹캠 옆)로 저장되어, 라이브에서 웹캠이 겹쳐 보이던 배치와 다릅니다.
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
| Noto Sans KR (본문 글꼴) | [Google Fonts](https://fonts.google.com/noto/specimen/Noto+Sans+KR), SIL OFL 1.1 |
| Chakra Petch (영문 제목 글꼴) | [Google Fonts](https://fonts.google.com/specimen/Chakra+Petch), SIL OFL 1.1 |
| JetBrains Mono (숫자 글꼴) | [Google Fonts](https://fonts.google.com/specimen/JetBrains+Mono), SIL OFL 1.1 |
| 욕설 목록 (`backend/data/profanity_ko.txt`) | [LDNOOBW List of Dirty, Naughty, Obscene, and Otherwise Bad Words](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words) `ko`, CC BY 4.0 — 오탐 항목 제거·일부 추가 |
| 흔한 비밀번호 상위 100 (`backend/data/common_passwords.txt`) | [SecLists](https://github.com/danielmiessler/SecLists) `xato-net-10-million-passwords-100.txt`, MIT |
| 로고·마스코트·배너·배경 텍스처 (`frontend/public/brand/`) | **AI 생성 이미지**를 가공해 사용 (원본·가공본은 `docs/design-refs/`) |
| livekit-client (WebRTC 송출·시청 SDK) | [livekit/client-sdk-js](https://github.com/livekit/client-sdk-js), Apache-2.0, npm |
| @tosspayments/tosspayments-sdk (토스 결제위젯 v2) | [npm](https://www.npmjs.com/package/@tosspayments/tosspayments-sdk) · [문서](https://docs.tosspayments.com/sdk/v2/js), MIT |
