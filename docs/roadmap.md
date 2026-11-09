# Roadmap

> 메인 세션만 이 문서를 갱신한다. 상태: ⬜ 대기 · 🟨 진행 · ✅ 완료 · ✂️ 제외

마감: 2026-11-19(목) 23:59 KST. 11/13까지 필수 기능 미완료 시 확장 단계부터 자른다(T30 → T29 → T28 → T27, 사용자 승인). 11/19는 기능 동결.

## 사용자 준비물
> 사용자만 할 수 있는 일. 기한 안에 안 되면 해당 task만 멈추고 나머지는 계속한다(AGENTS §9-6).

| 항목 | 필요한 task | 기한 | 상태 |
|---|---|---|---|
| **Rust(rustup) + MSVC Build Tools 설치** (관리자 권한) | T01 | 지금 | ⬜ |
| **Claude 권한 허용 목록 승인** (cargo·npm·git·gh·docker 등, 무인 실행용) | 전부 | 지금 | ⬜ |
| Oracle ARM 인스턴스 생성 (3 OCPU / 18GB, Ubuntu) + OCI Security List 80/443 개방 | T07 | 11/10 | ⬜ |
| 내도메인 `flutch.kro.kr` 발급 → 인스턴스 IP 연결 | T07 | 11/10 | ⬜ |
| GHCR 이미지 pull용 토큰(`read:packages`) + 서버 SSH 키 → GitHub Secrets | T07 | 11/10 | ⬜ |
| Oracle Object Storage 버킷 2개(`recordings` 공개 읽기·목록 비공개, `backups` 비공개) + Customer Secret Key(S3 호환) + namespace·region | T07 백업, T25 | 11/11 | ⬜ |
| LiveKit Cloud 가입, API key/secret, **대시보드 Settings › Project에서 Build 플랜 한도(분·Egress) 확인** | T08a 운영 확인 | 11/11 | ⬜ |
| LiveKit 대시보드에 웹훅 URL 등록 `https://flutch.kro.kr/api/v1/livekit/webhook` | T08b 운영 확인 | T07 배포 후 | ⬜ |
| 토스페이먼츠 개발자센터 가입, **결제위젯** 테스트 키(`test_gck`/`test_gsk`), 웹훅 URL 등록 | T21 | 11/14 | ⬜ |
| **수동 송출 확인 U1**: 배포 URL에서 PC 화면 공유(탭/시스템 소리) → 휴대폰 시청, 휴대폰 카메라 → PC 시청, 종료 | T15 | 11/12~13 | ⬜ |
| 카카오 개발자 앱(REST 키, Redirect URI `https://flutch.kro.kr/api/v1/auth/kakao/callback`) | T28 | 11/17 | ⬜ |
- 키·비밀값은 채팅에 붙여 넣지 않고 `.env`·GitHub Secrets에 직접 넣는다.
- 로컬 개발은 로컬 LiveKit 서버(compose.dev)로 진행하므로 LiveKit Cloud 키가 없어도 T08~T11 개발은 막히지 않는다.

## 사용자 확인 대기
> 문서에 없어 가정하고 진행한 결정(AGENTS §9-6). 사용자가 다르게 답하면 수정 task를 만든다. 메인 세션만 갱신.

| 날짜 | task | 가정한 내용 | 이유 | 바꾸면 영향 | 상태 |
|---|---|---|---|---|---|

## Task
> task 1개 = 서브에이전트 1개 = PR 1개. "선행"이 모두 ✅여야 착수. "(병렬)" 표시 task끼리 동시 진행 가능(최대 3). 정책은 "정책" 열의 `docs/features/` 문서.
> **T02에서 모든 테이블을 만든다.** 이후 task는 마이그레이션이 필요하면 ALTER만, 동시에 한 task만.

### 0. 기반 — 11/9~10
T01 → T02 → T02b → T03 → T04는 직렬. T05·T07은 T01 직후부터 이들과 병렬.

| ID | task | 정책 | 선행 | 상태 |
|---|---|---|---|---|
| T01 | 저장소 골격: backend(Axum health), frontend(Vite+Tailwind), **충돌 방지 골격**(모든 도메인 라우터 미리 연결, 도메인별 config·scheduler 등록·에러 enum 패턴), ts-rs 설정, compose.dev(Postgres + 로컬 LiveKit), `.env.example`, CI(Postgres service, `SQLX_OFFLINE`, fmt·clippy·test·ts-rs 비교·lint·typecheck·build, rust-cache), 로컬 sccache | architecture §3·§10 | — | ⬜ |
| T02 | DB 스키마 v1: **모든 테이블**(users, sessions, broadcasts, chats, bans·ban_events, follows, wallets·ledger_entries, payment_orders, donations, clutch_events·super_clutches, recordings·recording_views) | architecture §7 | T01 | ⬜ |
| T02b | **API 계약**: `docs/api.md`의 엔드포인트별 요청·응답·권한·도메인 에러 코드 채우기 (코드 없음, 문서만) | 전체 | T02 | ⬜ |
| T03 | 인증 백엔드: 가입·로그인·로그아웃·내 정보·닉네임/비밀번호 변경·테스트 로그인·시드·요청 제한·CSRF·Origin + **금지어·욕설·흔한 비밀번호 목록**(`backend/data/`, 출처 README) + 시나리오 | 01, 10 | T02b | ⬜ |
| T04 | 방송 CRUD·상태 전이·제한·목록/검색 API + 시나리오(재시작 후 조회 포함) | 02 | T03 | ⬜ |
| T05 | 디자인 토큰 + **DESIGN §9 공통 컴포넌트 전체** + 앱 레이아웃(헤더·사이드바) + 에러 화면 (병렬) | DESIGN | T01 | ⬜ |
| T07 | 배포: 서버 세팅·nginx·certbot·compose.prod·`ubuntu-24.04-arm` 러너 이미지 빌드·GHCR·SSH 배포·health 확인·DB 백업 (병렬, 서버·도메인 준비 후) | 10, architecture §10 | T01 | ⬜ |
| T06a | 인증 화면: 로그인·가입·약관 2장·테스트 로그인 버튼·내 정보 (병렬) | 01 | T03, T05 | ⬜ |
| T06b | 방송 화면: 등록·수정·**삭제**·**내 방송 관리**(READY 목록), 홈 그리드, **탐색·검색** (병렬) | 02 | T04, T05 | ⬜ |

### 1. 송출·시청 — 11/11~12
| ID | task | 정책 | 선행 | 상태 |
|---|---|---|---|---|
| T08a | LiveKit 기본: 토큰(송출·시청), 송출 시작/종료, 방 닫기, `publisher_identity` + 시나리오 `live_start_owner_only` `live_end_closes_room` | 03 | T04 | ⬜ |
| T08b | LiveKit 웹훅(서명)·60초 유예·기기 전환·3시간 제한·재시작 동기화·**시청자 수** + 나머지 live 시나리오 (병렬) | 03 | T08a | ⬜ |
| T08c | WebSocket 허브: 방송별 채널, Origin 검사, 쿠키 식별, **연결별 필터링**, 이벤트 형식 (병렬) | architecture §6 | T08a | ⬜ |
| T08d | 프론트 LiveKit 기반: `livekit-client` 설치, 연결·트랙 훅, Watch·Studio 라우트 자리, WebSocket 클라이언트 훅 | 03 | T08a, T08c, T06b | ⬜ |
| T09 | PC 송출 화면: 송출 전 확인·화면 공유·탭/시스템 소리·마이크·볼륨 슬라이더·레벨 미터·웹캠 모드·환경 안내·이어하기/기기 전환 모달 (병렬) | 03 | T08d | ⬜ |
| T10 | 모바일 송출 화면: 카메라 전환·음소거·Wake Lock (병렬) | 03 | T08d | ⬜ |
| T11 | 시청 화면: 플레이어·상태 4종(대기·재연결·화면 공유 중단·종료)·실시간 정보 변경·게이지 바 자리 (병렬) | 03 | T08d | ⬜ |

### 2. 필수 마무리 — 11/13
| ID | task | 정책 | 선행 | 상태 |
|---|---|---|---|---|
| T12 | 썸네일: 브라우저 1분 캡처 + `PUT /thumbnail` + 디스크 볼륨 + nginx `/media/thumbs` + 기본 썸네일 | 02 | T09, T07 | ⬜ |
| T13 | 모니터링: 메트릭·Alloy·Loki·Grafana 대시보드 (T12와 같은 배포 파일을 건드려서 직렬) | 10 | T12 | ⬜ |
| T14 | README 필수 4항목·지원 환경 표·수동 확인 절차 + **시나리오 이름 ↔ README 절차 대응표** (병렬) | spec 6 | T09, T10, T11 | ⬜ |
| T15 | 권한 시나리오 점검, **메인 세션이** `/ponytail-audit` 실행 후 must fix 반영 → **필수 완료** | 전체 | T12, T13, T14, U1 | ⬜ |

### 3·4·5. 선택 기능 — 11/14~17
T15 이후 **채팅(T16~) · 지갑(T20~) · 녹화(T25~) 세 줄을 동시에** 진행할 수 있다(최대 3).

| ID | task | 정책 | 선행 | 상태 |
|---|---|---|---|---|
| T16 | 채팅 백엔드: 작성·수정·삭제·커서 조회·욕설/금지어·도배 제한·차단 여부 확인 + 시나리오 | 04 | T15 | ⬜ |
| T17 | 채팅 화면 (비로그인 읽기 전용, 차단 입력창 포함) | 04 | T16 | ⬜ |
| T18 | 모더레이션 백엔드: 임시차단·영구 밴·이력·시청자 채팅 기록 API·관리 탭 + 시나리오 (병렬) | 05 | T16 | ⬜ |
| T18u | 모더레이션 화면: 채팅 닉네임 팝업(차단·채팅 기록), 시스템 메시지 | 05 | T17, T18 | ⬜ |
| T19 | 클러치 버튼(무료 콤보)·게이지 계산·게이지 바 + 시나리오 | 07 | T17 | ⬜ |
| T09b | 화면+웹캠 PIP (병렬) | 03, architecture §9 | T15 | ⬜ |
| T20 | 원장·지갑 핵심: 조건부 차감, 유형, 잔액 일치 + 시나리오 (병렬) | 06 | T15 | ⬜ |
| T20b | 회원 탈퇴: 잔액 소멸 원장 기록, LIVE 중 불가, 탈퇴 모달 + `auth_withdraw_forfeits_balance` | 01 | T20, T06a | ⬜ |
| T21 | 토스 결제: 주문·승인·웹훅(조회 API 재확인)·만료·"충전하고 후원하기"(max(부족분, 1,000))·종료 시 충전 전환 + 시나리오 | 06 | T20 | ⬜ |
| T22 | 후원 백엔드: 잔액 후원·익명·알림 이벤트(10초·스킵)·내역/합계·후원자 배지 + 시나리오 | 06 | T21 | ⬜ |
| T23 | 후원 화면: 내 지갑·충전 창·후원 창·**후원 알림·후원자 배지·이 방송 후원 목록**·결제 결과·스트리머 후원 통계 | 06 | T22 | ⬜ |
| T24 | 유료 클러치·SUPER BURST + 시나리오 | 07 | T19, T22 | ⬜ |
| T25 | 녹화: Egress → Object Storage, 처리 중/실패, 보관 정리(30일·16GB, `size_bytes`), 방송 삭제 시 객체 삭제 + 시나리오 (병렬) | 08 | T15 | ⬜ |
| T26 | 다시보기 목록·상세·플레이어·조회수·스트리머 페이지(다시보기 탭) | 08, 09 | T25 | ⬜ |

### 6. 확장 — 11/18 (시간이 되는 만큼, 위에서부터)
| ID | task | 정책 | 선행 | 상태 |
|---|---|---|---|---|
| T27 | 팔로우·사이드바 PULSE·PULSE 페이지 | 09 | T26 | ⬜ |
| T28 | 카카오 로그인 | 01 | T06a | ⬜ |
| T29 | SUPER CLUTCH 회전(30초·5분)·메인 히어로(`/home/hero`, 10초 폴링) + 시나리오 | 07 | T24 | ⬜ |
| T30 | CLUTCH FEED 가상 클립 | 07 | T29, T25 | ⬜ |

### 7. 제출 — 11/19
| ID | task | 정책 | 선행 | 상태 |
|---|---|---|---|---|
| T31 | `/ponytail-audit`·`/ponytail-debt` → README 알려진 한계, gitleaks, 시연 확인, 공개 전환(승인), 제출(승인) | spec | 전부 | ⬜ |

자를 순서(일정 부족 시, 사용자 승인): T30 → T29 → T28 → T27.
