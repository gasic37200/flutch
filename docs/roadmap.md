# Roadmap

> 메인 세션만 이 문서를 갱신한다. 상태: ⬜ 대기 · 🟨 진행 · ✅ 완료 · ✂️ 제외

마감: 2026-11-19(목) 23:59 KST. 11/13까지 필수 기능 미완료 시 확장 단계부터 자른다(T30 → T29 → T28 → T27, 사용자 승인). 11/19는 기능 동결.

## 사용자 준비물
> 사용자만 할 수 있는 일. 기한 안에 안 되면 해당 task만 멈추고 나머지는 계속한다(AGENTS §9-6).

| 항목 | 필요한 task | 기한 | 상태 |
|---|---|---|---|
| **개발 환경**: Windows 스마트 앱 컨트롤이 빌드한 실행 파일을 막아서 **WSL2 Ubuntu-24.04**(`~/flutch`, Rust·Node·gh·Docker 연동, SSH `localhost:2222`)로 구성 | T01 | 지금 | ✅ |
| **Claude 권한 허용 목록 승인** (cargo·npm·git·gh·docker 등, 무인 실행용) | 전부 | 지금 | ✅ |
| Oracle ARM 인스턴스 생성 (3 OCPU / 18GB, Ubuntu) + OCI Security List 80/443 개방 — `flutch-server-prod-01` 146.56.116.48, Docker·iptables 80/443 완료 | T07 | 11/10 | ✅ |
| 내도메인 `flutch.kro.kr` 발급 → 인스턴스 IP 연결 (A 146.56.116.48 확인) | T07 | 11/10 | ✅ |
| GHCR 이미지 pull용 토큰(`read:packages`) + 서버 SSH 키 → GitHub Secrets | T07 | 11/10 | ⬜ |
| **GitHub Secrets 4개**: `DEPLOY_HOST`(146.56.116.48) · `DEPLOY_USER`(ubuntu) · `DEPLOY_SSH_KEY`(서버 개인키 전체) · `GHCR_READ_TOKEN`(read:packages PAT). 넣은 뒤 Actions › Deploy › Run workflow(main) | T07 완료 확인 | 11/10 | ⬜ |
| **HTTPS 인증서**: Let's Encrypt가 `kro.kr` 전체 주간 50개 한도로 거절(11/09 18:30·18:46). 서버에서 `cd ~/flutch && ./init-cert.sh` 재시도(22:11 KST 이후). **서버 cron이 5분마다 자동 재시도**(11/10 09:4x 30분→5분, 사용자 승인)(`~/flutch/cert-retry.sh`, 로그 `backups/cert.log`, 성공 시 `.cert-ok` 남기고 멈춤 — 사용자 승인 11/09 19:3x) | T07 완료 확인 | 11/10 | ⬜ |
| Oracle Object Storage 버킷 2개(`recordings` 공개 읽기·목록 비공개, `backups` 비공개) + Customer Secret Key(S3 호환) + namespace·region | T07 백업, T25 | 11/11 | ⬜ |
| **녹화 저장소 연결(T25 이후)**: `recordings` 버킷 공개 읽기 + CORS(`https://flutch.kro.kr` GET·HEAD) → 서버 `.env`에 `RECORDING_S3_ENDPOINT`(`https://<ns>.compat.objectstorage.<region>.oraclecloud.com`)·`RECORDING_S3_BUCKET`·`RECORDING_S3_REGION`·`RECORDING_S3_ACCESS_KEY`·`RECORDING_S3_SECRET_KEY`·`RECORDING_PUBLIC_BASE_URL`(`https://objectstorage.<region>.oraclecloud.com/n/<ns>/b/<bucket>/o`). 하나라도 비면 녹화 꺼짐(방송은 정상). LiveKit Cloud Egress 한도 확인 | T25 운영 확인 | 배포 후 | ⬜ |
| LiveKit Cloud 가입, API key/secret, **대시보드 Settings › Project에서 Build 플랜 한도(분·Egress) 확인** | T08a 운영 확인 | 11/11 | ⬜ |
| LiveKit 대시보드에 웹훅 URL 등록 `https://flutch.kro.kr/api/v1/livekit/webhook` | T08b 운영 확인 | T07 배포 후 | ⬜ |
| 토스페이먼츠 개발자센터 가입, **결제위젯** 테스트 키 → 서버 `.env`에 `TOSS_CLIENT_KEY=test_gck_…`·`TOSS_SECRET_KEY=test_gsk_…`, 웹훅 URL `https://flutch.kro.kr/api/v1/payments/webhook`(이벤트 `PAYMENT_STATUS_CHANGED`) 등록 | T21 운영 확인 | 11/14 | ⬜ |
| **수동 송출 확인 U1**: 배포 URL에서 PC 화면 공유(탭/시스템 소리) → 휴대폰 시청, 휴대폰 카메라 → PC 시청, 종료. 함께 볼 것: 송출 탭을 백그라운드로 두고 1~2분 뒤 홈 카드 썸네일 갱신(T12), 모바일 Wake Lock·후면 카메라(T10), 소리 켜기·볼륨·전체화면(T11) | T15 | 11/12~13 | ⬜ |
| 카카오 개발자 앱(REST 키, Redirect URI `https://flutch.kro.kr/api/v1/auth/kakao/callback`) | T28 | 11/17 | ⬜ |
- 키·비밀값은 채팅에 붙여 넣지 않고 `.env`·GitHub Secrets에 직접 넣는다.
- 로컬 개발은 로컬 LiveKit 서버(compose.dev)로 진행하므로 LiveKit Cloud 키가 없어도 T08~T11 개발은 막히지 않는다.

## 사용자 확인 대기
> 문서에 없어 가정하고 진행한 결정(AGENTS §9-6). 사용자가 다르게 답하면 수정 task를 만든다. 메인 세션만 갱신.

| 날짜 | task | 가정한 내용 | 이유 | 바꾸면 영향 | 상태 |
|---|---|---|---|---|---|
| 2026-11-09 | T01 | 프론트 린터를 ESLint 대신 **oxlint**(`--deny-warnings`, any·console 금지·hooks 규칙) | Vite 최신 템플릿 기본값, 빠름 | ESLint로 교체 task 1개(설정만) | 대기 |
| 2026-11-09 | T01 | `Origin` 헤더가 **없는** 변경 요청도 403 | CSRF 보수적 처리(브라우저는 항상 보냄) | curl·외부 도구 호출에 Origin 필요. 바꾸면 middleware 한 줄 | 대기 |
| 2026-11-09 | T01 | 프론트 경로 `/` `/explore` `/live/:broadcastId` `/studio` `/replay/:recordingId` `/users/:userId` `/wallet` `/login` `/signup` | 문서에 경로 이름 없음 | router.tsx·링크 수정 | 대기 |
| 2026-11-09 | T01 | 로컬 compose.dev Postgres 호스트 포트 **5433**(이 PC의 5432는 다른 프로젝트가 사용) | 포트 충돌 | `.env`의 `POSTGRES_PORT`만 변경 | 대기 |
| 2026-11-09 | T02 | 닉네임 중복은 **대소문자 무시**(`lower(nickname)`), 탈퇴 30일 보류는 앱에서 확인 | 비슷한 닉네임 사칭 방지 | 인덱스 한 줄 교체 마이그레이션 | 대기 |
| 2026-11-09 | T02 | 로그인 잠금 상태를 DB에 저장(`failed_login_count`·`locked_until`) | 재시작해도 잠금 유지 | 메모리로 옮기면 컬럼 미사용 | 대기 |
| 2026-11-09 | T02 | 카테고리 8개는 DB CHECK 없이 코드 상수 | 목록 변경이 잦을 수 있음 | 없음 | 대기 |
| 2026-11-09 | T02b(메인) | JSON 필드는 에러 본문 포함 **전부 camelCase**(`requestId`, `nextCursor` 등) | conventions §1과 api.md 초안 불일치 정리 | T03에서 error.rs 필드 이름 변경 | 대기 |
| 2026-11-09 | T05 | 좁은 화면에서 오른쪽 패널은 숨기지 않고 **본문 아래로 쌓음**, 사이드바는 lg 미만에서 햄버거 서랍 | 숨기면 채팅이 사라짐(DESIGN §7) | AppShell·ShellBody 수정 | 대기 |
| 2026-11-09 | T05 | 비로그인도 방송하기 아이콘은 `/studio`로 이동, 로그인 요구는 스튜디오 화면에서 | 진입점 단순화 | 헤더 링크 조건 추가 | 대기 |
| 2026-11-09 | T05 | 글꼴은 CDN(Google Fonts·jsDelivr), 헤더 메뉴의 "내 정보"는 화면이 생기면 추가 | 의존성 없이 적용 | CSP 도입 시 도메인 허용 필요 | 대기 |
| 2026-11-09 | T07 | 배포는 workflow가 deploy 파일 3개를 scp로 서버에 올림(서버에 저장소 clone 없음) | 비공개 저장소 | 없음 | 대기 |
| 2026-11-09 | T07 | DB 백업 파일명을 요일로 해 7개 순환(별도 삭제 로직 없음) | 7일 보관을 가장 단순하게 | 없음 | 대기 |
| 2026-11-09 | T02b | 방송 소유자는 자기 방송에 후원·클러치·버스트 불가(`DONATION_SELF`·`CLUTCH_SELF`) | 게이지·후원 집계 부풀리기 방지 | 에러 코드 제거 | 대기 |
| 2026-11-09 | T02b | 잔액 후원 API에 멱등 키 없음(두 번 누르면 2건, 원장 정합성은 유지) → 프론트 버튼 비활성화 | 결제 확정은 멱등, 잔액 후원은 사용자 동작 | 요청 키 컬럼·검사 추가 | 대기 |
| 2026-11-09 | T02b | 카테고리 코드 8개 `just_chatting·game·fps·rpg·horror·virtual·sports·music`, 썸네일 JPEG 512KB, 검색어 1~30자, LiveKit 토큰 TTL 4시간 | 문서에 수치 없음 | api.md·상수 수정 | 대기 |
| 2026-11-09 | T02b | 후원 목록은 donation·burst만, 유료 클러치는 합계(`clutchTotal`)로만. 후원자 배지는 donation·burst 이력 기준 | 1비트 클러치가 목록을 덮음 | 응답 필드 변경 | 대기 |
| 2026-11-09 | T03 | 세션 쿠키는 로컬 http에서도 항상 `Secure`(브라우저가 http://localhost는 받아 줌) | 운영과 같은 설정 유지 | LAN IP로 http 개발 시 쿠키 안 붙음 | 대기 |
| 2026-11-09 | T03 | `SEED_PASSWORD` 비우면 시드 계정 비밀번호 랜덤(테스트 로그인 버튼으로만 진입) | 운영 비밀번호 노출 방지 | README에 심사용 비밀번호 적으려면 서버 .env에 값 설정 | 대기 |
| 2026-11-09 | T03 | 닉네임 예약어 10개(운영자·관리자·flutch·admin·system·탈퇴한사용자·익명의후원자 등, 포함만 돼도 차단), 욕설 목록은 LDNOOBW ko(CC BY 4.0)에서 오탐 큰 13개 제외 | 일상어 오탐 방지 | `backend/data/` 목록 수정 | 대기 |
| 2026-11-09 | T03 | 로그인 5번째 실패 응답부터 `AUTH_LOCKED`, 가입 IP 제한은 422가 아닌 시도만 셈 | 입력 실수로 막히지 않게 | handlers 한두 줄 | 대기 |
| 2026-11-09 | T04 | 방송 목록 정렬은 지금 id DESC(시청자 수 정렬은 T08b에서), 커서는 마지막 id | 시청자 수가 DB에 없음 | T08b에서 정렬 교체 | 대기 |
| 2026-11-09 | T04 | `thumbnailUrl` = `/media/thumbs/{id}.jpg?v=<thumbnail_path>` — T12가 `thumbnail_path`에 버전 문자열 저장 | 캐시 무효화 | T12 저장 형식 | 대기 |
| 2026-11-09 | T04 | LiveKit 서버 API는 livekit-api crate 대신 HS256 JWT 직접 서명 + Twirp 호출(`hmac` 추가) | crate가 protobuf·aws-lc까지 끌고 옴 | crate 도입 시 livekit.rs 교체 | 대기 |
| 2026-11-09 | T06a | 프론트 경로 `/terms` `/privacy` `/me` | 문서에 경로 없음 | router·링크 수정 | 대기 |
| 2026-11-09 | T06b | 프론트 경로 `/manage`(내 방송 관리) `/broadcasts/new` `/broadcasts/:id/edit`, 스튜디오 진입 `/studio?broadcast=<id>`, 등록·수정 후 `/manage`로 이동 | 문서에 경로 없음 | router·링크 수정 | 대기 |
| 2026-11-09 | T06b | 방송 등록·수정 폼에서 PIP 위치 제외(스튜디오 T09에서 설정) | PIP는 송출 중 조작 | 폼 필드 추가 | 대기 |
| 2026-11-09 | T08a | "다른 기기 송출 중" 판정은 DB(`publisher_identity`+`disconnected_at`)만 사용, LiveKit 참가자 조회 안 함 | 불확실하면 확인창 쪽으로, LiveKit 장애 무관 | T08b 웹훅 전까지 탭을 닫고 새 탭에서 시작해도 전환 확인창이 뜸 | 대기 |
| 2026-11-09 | T08a | 시청 토큰 요청에 요청 제한 없음 | 문서에 없음 | 제한 추가 시 handler 한 줄 | 대기 |
| 2026-11-09 | T08c | WebSocket은 Origin 검사(403)를 방송 존재 확인(404)보다 먼저, 소유자 여부는 연결 시점에 한 번 판정 | CSWSH 방어를 DB 조회 전에 | 연결 중 로그아웃해도 그 연결은 소유자형 이벤트 유지 | 대기 |
| 2026-11-09 | T08c | 방송별 채널 크기 256, ping 30초(다음 주기까지 pong 없으면 끊음) | 문서에 수치 없음 | realtime.rs 상수 | 대기 |
| 2026-11-09 | T08d | 송출 연결이 LiveKit 재연결을 포기하면 자동 재시도 없이 "다시 연결" 버튼(같은 신원이라 서버가 이어서 송출로 처리) | 무한 재시도 방지 | usePublisher에 재시도 루프 추가 | 대기 |
| 2026-11-09 | T08d | WebSocket 403(Origin)은 브라우저가 코드를 안 줘서 연속 5회 실패 시 재접속 중단 | 브라우저 한계 | 상한 조정 | 대기 |
| 2026-11-09 | T09 | 마이크·화면 소리 볼륨 0~150%(기본 100), 음소거는 GainNode 0(트랙 유지, 무음 전송), 화면 모드는 소리 없는 화면도 무음 오디오 트랙 게시 | 소스 변경 시 끊김 없게 | useStudioMedia 수정 | 대기 |
| 2026-11-09 | T09 | 송출 화면에서 제목·카테고리 실시간 수정 없음(읽기 전용 패널) — 수정은 /manage에서 | 화면 이동 시 송출이 끊김 | 스튜디오에 인라인 폼 추가 | 대기 |
| 2026-11-09 | T11 | 시청 연결 직후 2초는 "화면 공유 중단" 대신 "연결하는 중" 표시, 소리 다시 끄기는 볼륨 0으로 | 트랙 구독 지연 | 상수·버튼 추가 | 대기 |
| 2026-11-09 | T08b | LiveKit `room_finished`는 종료가 아니라 60초 유예 시작으로 처리 | 빈 방이 약 18초 만에 닫혀 그대로 종료하면 유예 규칙이 깨짐(실측) | 웹훅 핸들러 분기 | 대기 |
| 2026-11-09 | T08b | 3시간 10분 전 경고 중복 방지는 메모리(재시작 시 1회 더 갈 수 있음), LIVE 목록 커서는 순위 위치(offset) | 단순화, LIVE 수 수십 개 | DB 컬럼 추가·커서 방식 변경 | 대기 |
| 2026-11-09 | T08b | 유예·최대 길이를 환경변수 `LIVE_GRACE_SECS`·`LIVE_MAX_DURATION_SECS`로 조정 가능(기본 60·10800) | 수동 확인용 | 없음 | 대기 |
| 2026-11-10 | T10 | 모바일 판정은 UA(Android·iPhone·iPad·Mobile) — 태블릿도 모바일 화면, 전후면 전환은 기존 트랙을 끄고 새로 잡음(시청자 쪽 짧은 끊김 가능) | 모바일은 카메라 동시 2개를 못 여는 기기가 있음 | replaceTrack 적용 시 무중단 | 대기 |
| 2026-11-10 | T05d | 글꼴을 시안대로 Pretendard → **Noto Sans KR**, 잔액 단위는 시안의 "Beats" 대신 "비트"(decisions §1) | 시안 우선, 화폐 표기는 decisions 우선 | BeatAmount 한 줄 | 대기 |
| 2026-11-10 | T05d | 툴팁은 아래로 열림(헤더 맨 위에서 위로 열면 잘림), 비로그인 방송하기 아이콘은 `/login`으로 | 화면 밖 잘림 방지 | Tooltip·Header 한 줄 | 대기 |
| 2026-11-10 | T14 | README 시청 지원 브라우저를 "최신 Chrome·Edge·Safari·Firefox"로 기재 | 문서에 시청 지원 범위 없음 | README 한 줄 | 대기 |
| 2026-11-10 | T12 | 썸네일 버전 문자열은 갱신 시각(ms), 숨김(DELETE) 때만 파일 삭제·ENDED 파일은 다시보기용으로 유지, 서버는 JPEG 시그니처만 확인(디코딩 안 함) | 단순화 | 정리 작업·검증 추가 | 대기 |
| 2026-11-10 | T11b | 시청 화면 재연결 상태에 시안의 경과 타이머 대신 "60초 안에 돌아오지 않으면 방송이 종료돼요"만(상세 응답에 시작 시각 없음), 종료 화면 두 번째 버튼을 "홈으로" → "스트리머 페이지", 모바일 "후원" 탭은 T23 전까지 숨김 | 데이터 없음·시안 우선·"준비 중" 금지 | Broadcast에 since 추가 시 타이머 | 대기 |
| 2026-11-10 | T06c | 홈 메인 라이브는 시청자 수 1위의 썸네일 카드(자동재생 아님, 게이지 1위·CLUTCH TIME 패널은 T29), 로그인·가입은 앱 셸 안 2단, 방송 폼은 "등록"·"취소" 유지(시안의 "등록하고 스튜디오로"·"대기 방송으로 저장" 2버튼 아님) | 기능 범위·공용 router 변경 회피 | HomePage·router·BroadcastForm 수정 | 대기 |
| 2026-11-10 | T09c | 스튜디오: 방송 정보 수정·마이크 장치 선택·후원 알림·PIP는 숨김, 3시간 자동 종료는 고정 안내 문구, 볼륨 0~150% 유지, 시안의 STUDIO 전용 헤더 대신 앱 셸 헤더 | 기능 없음·동작 보존 | 각 기능 task에서 추가 | 대기 |
| 2026-11-10 | T13 | Grafana 관리자 사용자명 `admin`, 비밀번호는 서버 `.env`의 `GRAFANA_ADMIN_PASSWORD`(메인 세션이 랜덤 생성해 넣음), 이미지 버전 고정(prometheus v3.5.0·loki 3.5.5·alloy v1.10.0·node-exporter v1.9.1·grafana 12.1.0) | 재현 가능한 배포 | compose 버전 교체 | 대기 |
| 2026-11-10 | T15c(메인) | **시드(테스트) 계정은 비밀번호·닉네임 변경 불가**(403) | 심사자가 공용 계정을 같이 쓰므로 한 명이 바꾸면 다른 심사자의 세션이 끊김 | features/01·api.md 한 줄, handler 분기 | 대기 |
| 2026-11-10 | T15a | 송출 시작 직후 송출자가 LiveKit에 붙기 전 몇 초 동안 `reconnecting=true`(시청 화면에 "재연결 중"이 잠깐 보임), 그 사이 다른 기기가 시작하면 확인 없이 전환 | 미접속 시 유예를 시작하려고, 컬럼 추가 없이 | 별도 상태 컬럼 추가 | 대기 |
| 2026-11-10 | T16 | 채팅 검사 순서 방송 상태 409 → 차단 403 → 본문 422 → 도배 429, 수정에는 1초·반복 제한 없음, 차단 사용자도 자기 글 삭제 가능 | 문서에 순서 없음 | handler 순서 | 대기 |
| 2026-11-10 | T20 | 지갑 내역에서 숨긴 방송 제목은 "삭제된 방송", 같은 주문 재적립 신호는 `PAYMENT_ALREADY_CONFIRMED`(T21이 200 재응답으로 바꿈), 지갑 행이 없으면 잔액 0 | 원래 제목 숨김·멱등 | 응답 문구·코드 | 대기 |
| 2026-11-10 | T25 | Egress는 송출자 첫 접속 웹훅에서 시작(방이 생긴 뒤), 객체 키 `recordings/{broadcastId}.mp4`, 크기 0이면 failed, 유예 중 방이 다시 만들어지면 앞 구간만 저장(자동 재시도 없음), 삭제는 상태 먼저 → 파일 삭제 재시도 | Egress는 방이 있어야 시작됨 | 재시작 로직 추가 | 대기 |
| 2026-11-10 | T26 | 다시보기 재생은 브라우저 기본 `<video controls>`(배속·전체화면 포함), 스트리머 페이지 탭은 "방송"·"다시보기" 2개(관리 탭은 T18u·T23 전까지 숨김, 본인은 "내 방송 관리" 링크), 팔로워 수는 표시·팔로우 버튼은 T27 | 동작하지 않는 탭은 숨김 | 탭 추가 | 대기 |
| 2026-11-10 | T21 | 확정된 주문 재시도는 만료보다 먼저 판단(만료 뒤에도 같은 결과), 토스가 DONE이면 만료된 주문도 적립, 승인 실패 시 주문은 pending 유지, notice 3종(방송 종료·밴·잔액 부족), orderName "N비트 충전"·"N비트 충전 후 후원" | 돈이 나간 결제는 반드시 적립 | 문구·상태값 | 대기 |
| 2026-11-10 | T17 | 채팅 메시지 메뉴는 "⋯" 버튼으로 줄 아래 펼침(스크롤 안 드롭다운 잘림 회피), 시각은 마우스 올릴 때만(터치 기기 숨김), 모바일 송출 오버레이는 읽기 전용 최근 5개 | 스크롤 영역 제약 | 컴포넌트 수정 | 대기 |
| 2026-11-10 | T18 | 차단 덮어쓰기는 새 행(목록 최상단), 없는 차단 해제는 이력 안 남김, 영구 밴 시 채팅 일괄 삭제는 현재 LIVE 방송만·알림은 `chat.system` 하나로 | 단순화 | 이력·삭제 범위 | 대기 |
| 2026-11-10 | T19 | 유료 클러치 후원 기록은 익명·메시지 없음, 무료 콤보 표시는 마지막 클릭 후 30초 타이머로 근사(서버는 슬라이딩 창), 5분 창 감소는 이벤트가 없어 시청 화면이 20초마다 재조회, SUPER CLUTCH 연출은 로고 + 그라데이션으로 단순 구현(시안 정밀 대조 안 함) | 단순화 | 연출·주기 | 대기 |
| 2026-11-10 | T18u | 차단 해제 알림 이벤트가 없어 차단된 입력창은 차단 중에만 10초마다 상세 재조회, 영구 밴 해제 버튼은 항상 노출(DELETE 멱등), 모바일 송출 오버레이에는 닉네임 팝업 없음 | 이벤트 추가 회피 | `chat.system` kind 확장 시 폴링 제거 | 대기 |
| 2026-11-10 | T22 | 후원 통계는 송출한 적 있는 방송만 id 최신순, last7Days는 KST 달력 기준 오늘 포함 7일, 스킵은 방송 상태와 무관, 목록에서 후원자 본인이라도 소유자가 아니면 익명 donor는 null | 문서에 기준 없음 | 집계 기준 | 대기 |
| 2026-11-10 | T09b | PIP는 PC 화면 모드에서만(모바일 송출엔 없음), 웹캠 360p30, 모바일 시청은 PIP 탭으로 숨기고 컨트롤 바 카메라 버튼으로 다시 보이기, 웹캠만 모드로 바꾸면 PIP 자동 꺼짐 | 대역폭·화면 크기 | 컴포넌트 수정 | 대기 |

## Task
> task 1개 = 서브에이전트 1개 = PR 1개. "선행"이 모두 ✅여야 착수. "(병렬)" 표시 task끼리 동시 진행 가능(최대 3). 정책은 "정책" 열의 `docs/features/` 문서.
> **모델·사고**: 서브에이전트를 띄울 때 Agent 도구의 `model`·`effort`에 그대로 넣는다(AGENTS §9-9). 같은 task를 다시 맡길 때는 사고 단계를 한 단계 올린다.
> **T02에서 모든 테이블을 만든다.** 이후 task는 마이그레이션이 필요하면 ALTER만, 동시에 한 task만.

### 0. 기반 — 11/9~10
T01 → T02 → T02b → T03 → T04는 직렬. T05·T07은 T01 직후부터 이들과 병렬.

| ID | task | 정책 | 선행 | 모델·사고 | 상태 |
|---|---|---|---|---|---|
| T01 | 저장소 골격: backend(Axum health), frontend(Vite+Tailwind), **충돌 방지 골격**(모든 도메인 라우터 미리 연결, 도메인별 config·scheduler 등록·에러 enum 패턴), ts-rs 설정, compose.dev(Postgres + 로컬 LiveKit), `.env.example`, CI(Postgres service, `SQLX_OFFLINE`, fmt·clippy·test·ts-rs 비교·lint·typecheck·build, rust-cache), 로컬 sccache | architecture §3·§10 | — | Opus · high | ✅ |
| T02 | DB 스키마 v1: **모든 테이블**(users, sessions, broadcasts, chats, bans·ban_events, follows, wallets·ledger_entries, payment_orders, donations, clutch_events·super_clutches, recordings·recording_views) | architecture §7 | T01 | Opus · high | ✅ |
| T02b | **API 계약**: `docs/api.md`의 엔드포인트별 요청·응답·권한·도메인 에러 코드 채우기 (코드 없음, 문서만) | 전체 | T02 | Opus · medium | ✅ |
| T03 | 인증 백엔드: 가입·로그인·로그아웃·내 정보·닉네임/비밀번호 변경·테스트 로그인·시드·요청 제한·CSRF·Origin + **금지어·욕설·흔한 비밀번호 목록**(`backend/data/`, 출처 README) + 시나리오 | 01, 10 | T02b | Opus · high | ✅ |
| T04 | 방송 CRUD·상태 전이·제한·목록/검색 API + 시나리오(재시작 후 조회 포함) | 02 | T03 | Sonnet · medium | ✅ |
| T05 | 디자인 토큰 + **DESIGN §9 공통 컴포넌트 전체** + 앱 레이아웃(헤더·사이드바) + 에러 화면 (병렬) | DESIGN | T01 | Sonnet · medium | ✅ |
| T07 | 배포: 서버 세팅·nginx·certbot·compose.prod·`ubuntu-24.04-arm` 러너 이미지 빌드·GHCR·SSH 배포·health 확인·DB 백업 (병렬, 서버·도메인 준비 후) | 10, architecture §10 | T01 | Sonnet · medium | 🟨 |
| T06a | 인증 화면: 로그인·가입·약관 2장·테스트 로그인 버튼·내 정보 (병렬) | 01 | T03, T05 | Sonnet · medium | ✅ |
| T06b | 방송 화면: 등록·수정·**삭제**·**내 방송 관리**(READY 목록), 홈 그리드, **탐색·검색** (병렬) | 02 | T04, T05 | Sonnet · medium | ✅ |
| T05d | **디자인 맞춤 · 공통**: 시안 보드 `Main`·`Components`·`Guest`·`Errors`(docs/design-refs/boards)에 토큰·공통 컴포넌트·헤더·사이드바·비로그인 상태·에러 화면을 맞춤 (11/10 추가 — T05가 시안 아티팩트를 열지 못해 home.webp만 보고 만듦) | DESIGN, boards | T05 | Sonnet · medium | ✅ |
| T06c | **디자인 맞춤 · 홈·탐색·방송 폼·인증**: 보드 `Home`·`HomeMobile`·`Explore`·`BroadcastForm`·`Auth`·`Signup`·`SmallScreens`(내 정보·약관) (병렬) | DESIGN, boards | T05d | Sonnet · medium | ✅ |

### 1. 송출·시청 — 11/11~12
| ID | task | 정책 | 선행 | 모델·사고 | 상태 |
|---|---|---|---|---|---|
| T08a | LiveKit 기본: 토큰(송출·시청), 송출 시작/종료, 방 닫기, `publisher_identity` + 시나리오 `live_start_owner_only` `live_end_closes_room` | 03 | T04 | Opus · high | ✅ |
| T08b | LiveKit 웹훅(서명)·60초 유예·기기 전환·3시간 제한·재시작 동기화·**시청자 수**(WebSocket으로 발행) + 나머지 live 시나리오 (T08d와 병렬) | 03 | T08a, T08c | Opus · high | ✅ |
| T08c | WebSocket 허브: 방송별 채널, Origin 검사, 쿠키 식별, **연결별 필터링**, 이벤트 형식 | architecture §6 | T08a | Opus · high | ✅ |
| T08d | 프론트 LiveKit 기반: `livekit-client` 설치, 연결·트랙 훅, Watch·Studio 라우트 자리, WebSocket 클라이언트 훅 | 03 | T08a, T08c, T06b | Sonnet · medium | ✅ |
| T09 | PC 송출 화면: 송출 전 확인·화면 공유·탭/시스템 소리·마이크·볼륨 슬라이더·레벨 미터·웹캠 모드·환경 안내·이어하기/기기 전환 모달 (병렬) | 03 | T08d | Sonnet · medium | ✅ |
| T10 | 모바일 송출 화면: 카메라 전환·음소거·Wake Lock (병렬) | 03 | T08d | Sonnet · medium | ✅ |
| T11 | 시청 화면: 플레이어·상태 4종(대기·재연결·화면 공유 중단·종료)·실시간 정보 변경·게이지 바 자리 (병렬) | 03 | T08d | Sonnet · medium | ✅ |
| T11b | **디자인 맞춤 · 시청**: 보드 `Watch`·`WatchMobile`·`WatchStates` (병렬) | DESIGN, boards | T05d, T11 | Sonnet · medium | ✅ |

### 2. 필수 마무리 — 11/13
| ID | task | 정책 | 선행 | 모델·사고 | 상태 |
|---|---|---|---|---|---|
| T12 | 썸네일: 브라우저 1분 캡처 + `PUT /thumbnail` + 디스크 볼륨 + nginx `/media/thumbs` + 기본 썸네일 | 02 | T09, T07 | Sonnet · medium | ✅ |
| T09c | **디자인 맞춤 · 스튜디오**: 보드 `Studio`·`StudioPrelive`·`StudioMobile`·`SmallScreens`(스튜디오 모달) (스튜디오 파일을 건드려서 T12 뒤) | DESIGN, boards | T05d, T10, T12 | Sonnet · medium | ✅ |
| T13 | 모니터링: 메트릭·Alloy·Loki·Grafana 대시보드 (T12와 같은 배포 파일을 건드려서 직렬) | 10 | T12 | Sonnet · medium | ✅ |
| T14 | README 필수 4항목(서비스 주소 포함)·지원 환경 표·수동 확인 절차 + **시나리오 이름 ↔ README 절차 대응표** (병렬) | spec 6 | T07, T09, T10, T11 | Sonnet · medium | ✅ |
| T15a | **감사 수정 · 백엔드 송출**: 송출자 미접속 시 유예 시작(must), 구독 참가자가 유예를 풀지 않게(must), 시청자 hidden, 재접속 시청자 수, 이중 시작·유예 경합·종료 후 토큰 | audit 11/10 | T13 | Opus · high | ✅ |
| T15b | **감사 수정 · 프론트 송출·시청**: 시청 신원 분리(must), 서버 종료 시 캡처 해제, 시작 중 이탈 연결 누수, 카메라 전환, 재조회 실패 시 에러 화면, 종료 시간 멈춤 (병렬) | audit 11/10 | T13 | Sonnet · high | ✅ |
| T15c | **감사 수정 · 인증·운영**: 시드 계정 비밀번호·닉네임 잠금, 테스트 로그인 제한, HTTP 타임아웃, 메트릭 메서드 라벨, Loki level 라벨, 모니터링 재시작, 백업 체크섬, HSTS·Grafana 쿠키 (병렬) | audit 11/10 | T13 | Sonnet · medium | ✅ |
| T15 | 권한 시나리오 점검, **메인 세션이** `/ponytail-audit` 실행 후 must fix 반영 → **필수 완료** | 전체 | T12, T13, T14, T06c, T11b, T09c, T15a, T15b, T15c, U1 | 메인 세션 | 🟨 |

### 3·4·5. 선택 기능 — 11/14~17
T15 이후 **채팅(T16~) · 지갑(T20~) · 녹화(T25~) 세 줄을 동시에** 진행할 수 있다(최대 3).
> **예외(2026-11-10 11:5x, 사용자 승인)**: 필수 코드(T15a~c 포함)는 모두 머지됐고 남은 관문이 사용자 U1 수동 확인뿐이라, T15 완료 표시 전에 T16·T20·T25를 시작한다. U1에서 문제가 나오면 그 수정을 우선한다.

| ID | task | 정책 | 선행 | 모델·사고 | 상태 |
|---|---|---|---|---|---|
| T16 | 채팅 백엔드: 작성·수정·삭제·커서 조회·욕설/금지어·도배 제한·차단 여부 확인 + 시나리오 | 04 | T15 | Sonnet · medium | ✅ |
| T17 | 채팅 화면 (비로그인 읽기 전용, 차단 입력창 포함) | 04 | T16 | Sonnet · medium | ✅ |
| T18 | 모더레이션 백엔드: 임시차단·영구 밴·이력·시청자 채팅 기록 API·관리 탭 + 시나리오 (병렬) | 05 | T16 | Sonnet · medium | ✅ |
| T18u | 모더레이션 화면: 채팅 닉네임 팝업(차단·채팅 기록), 시스템 메시지 | 05 | T17, T18 | Sonnet · medium | ✅ |
| T19 | 클러치 버튼(무료 콤보)·게이지 계산·게이지 바 + 시나리오 | 07 | T17 | Sonnet · medium | ✅ |
| T09b | 화면+웹캠 PIP (시청·스튜디오 화면을 건드려서 채팅 화면 작업 뒤에) | 03, architecture §9 | T19 | Sonnet · medium | ✅ |
| T20 | 원장·지갑 핵심: 조건부 차감, 유형, 잔액 일치 + 시나리오 (병렬) | 06 | T15 | Opus · high | ✅ |
| T20b | 회원 탈퇴: 잔액 소멸 원장 기록, LIVE 중 불가, **시드(테스트) 계정은 탈퇴 불가(`AUTH_SEED_ACCOUNT_LOCKED`)**, 탈퇴 모달 + `auth_withdraw_forfeits_balance` | 01 | T20, T06a | Opus · high | 🟨 |
| T21 | 토스 결제: 주문·승인·웹훅(조회 API 재확인)·만료·"충전하고 후원하기"(max(부족분, 1,000))·종료 시 충전 전환 + 시나리오 | 06 | T20 | Opus · high | ✅ |
| T22 | 후원 백엔드: 잔액 후원·익명·알림 이벤트(10초·스킵)·내역/합계·후원자 배지 + 시나리오 | 06 | T21 | Opus · high | ✅ |
| T23 | 후원 화면: 내 지갑·충전 창·후원 창·**후원 알림·후원자 배지·이 방송 후원 목록**·결제 결과·스트리머 후원 통계 | 06 | T22 | Sonnet · medium | 🟨 |
| T24 | 유료 클러치·SUPER BURST + 시나리오 + **T19의 `clutch.gauge` 묶음 상태를 전역 static에서 AppState로 이동**(conventions §1 위반 정리) | 07 | T19, T22 | Opus · high | 🟨 |
| T25 | 녹화: Egress → Object Storage, 처리 중/실패, 보관 정리(30일·16GB, `size_bytes`), 방송 삭제 시 객체 삭제 + 시나리오 (병렬) | 08 | T15 | Sonnet · medium | ✅ |
| T26 | 다시보기 목록·상세·플레이어·조회수·스트리머 페이지(다시보기 탭) | 08, 09 | T25 | Sonnet · medium | ✅ |
| T26b | 스트리머 프로필 API `GET /users/{id}`(T26 화면이 쓰는데 백엔드에 없었음 — 확장 T27과 분리) | 09 | T26 | Sonnet · medium | ✅ |

### 6. 확장 — 11/18 (시간이 되는 만큼, 위에서부터)
| ID | task | 정책 | 선행 | 모델·사고 | 상태 |
|---|---|---|---|---|---|
| T27 | 팔로우·사이드바 PULSE·PULSE 페이지 | 09 | T26 | Sonnet · medium | ⬜ |
| T28 | 카카오 로그인 | 01 | T06a | Sonnet · medium | ⬜ |
| T29 | SUPER CLUTCH 회전(30초·5분)·메인 히어로(`/home/hero`, 10초 폴링) + 시나리오 | 07 | T24 | Opus · high | ⬜ |
| T30 | CLUTCH FEED 가상 클립 | 07 | T29, T25 | Sonnet · medium | ⬜ |

### 7. 제출 — 11/19
| ID | task | 정책 | 선행 | 모델·사고 | 상태 |
|---|---|---|---|---|---|
| T31 | `/ponytail-audit`·`/ponytail-debt` → README 알려진 한계, gitleaks, 시연 확인, 공개 전환(승인), 제출(승인) | spec | 전부 | 메인 세션 | ⬜ |

자를 순서(일정 부족 시, 사용자 승인): T30 → T29 → T28 → T27.
