# API 명세 (v1)

> 상태: **계약 v1** (T02b). 프론트·백엔드는 이 문서만 보고 병렬로 구현한다. 엔드포인트·필드를 바꾸는 PR은 이 문서를 같은 PR에서 갱신한다.
> 근거: `docs/features/01~10`, `docs/architecture.md`, `backend/migrations/*_schema_v1.sql`. 절 제목의 `(f02)`는 `features/02`.
> 에러 형식은 T03에서 이 문서의 camelCase(`requestId`·`retryAfterSec`)로 맞췄다.

## 0. 규칙
- 접두사 `/api/v1`. 리소스는 경로, 필터·검색·페이지는 쿼리, 데이터는 JSON 본문. **JSON 필드는 전부 camelCase**.
- enum 값은 DB 그대로: 방송 상태만 대문자(`READY`/`LIVE`/`ENDED`), 나머지는 소문자 snake(`bottom_right`, `charge` …).
- 인증: httpOnly 세션 쿠키 `flutch_session`. 로그인 필요 엔드포인트에 세션이 없거나 만료면 401 `UNAUTHENTICATED`.
- CSRF: POST/PUT/PATCH/DELETE는 `Origin`이 `ALLOWED_ORIGINS`에 있어야 한다. **없거나 다르면 403 `FORBIDDEN`** (웹훅 2개만 예외).
- 숫자 id·금액은 JSON number(i64). 시간은 RFC 3339 UTC(`Z`) 문자열, 화면은 KST로 표시.
- 문자열 길이는 앞뒤 공백을 자른 뒤 **유니코드 문자 수**로 센다.
- 페이지: 요청 `?cursor=<불투명 문자열>&limit=<기본 20, 최대 50>`, 응답 `{ "items": [...], "nextCursor": "…" | null }` (아래 `Page<T>`). 채팅만 `?before=<id>`(§4).
- 에러 본문 (응답 헤더 `x-request-id`도 같은 값, 429는 `Retry-After` 헤더도):
  ```json
  { "error": { "code": "BROADCAST_NOT_OWNER", "message": "방송 소유자만 할 수 있어요.", "requestId": "…",
               "fields": { "title": "2~50자로 입력해 주세요." }, "retryAfterSec": 30 } }
  ```
  `fields`는 `VALIDATION_FAILED`에만, `retryAfterSec`는 429와 `MODERATION_TIMED_OUT`에만 붙는다. 화면은 `message`를 그대로 보여준다.
  JSON 본문이 깨졌거나 필드 타입·누락이 맞지 않으면 422 `VALIDATION_FAILED` + `fields.body`(백엔드 `extract::JsonBody`).
- 상태 코드: 200/201/204 성공, 401 미인증, 403 권한 없음(소유자 아님·차단됨·Origin), 404 없음(숨김 방송 포함), 409 상태 충돌(잘못된 전이·LIVE 아님·잔액 부족·한도), 422 검증 실패(형식·금지어), 429 제한 초과, 502 외부 연동(토스) 실패, 500 `INTERNAL`.
- 검사 순서: 401 → 404 → 403 → 422 → 409. (권한 없는 사람에게 상태를 알려 주지 않도록 403이 409보다 먼저)

## 1. 공통 타입
TypeScript 표기. 실제 타입은 ts-rs 생성본(`frontend/src/api/generated/`)이 원본이고, 이름은 아래와 같게 만든다.

```ts
type UserSummary = { id: number; nickname: string }  // 탈퇴 사용자는 nickname "탈퇴한 사용자"
// 아바타(마스코트 표정 6 × 링 색 6)와 채팅 닉네임 색은 클라이언트가 id로 계산: face = id % 6, ring = Math.floor(id / 6) % 6

type Me = {
  id: number; loginId: string | null /* 카카오 계정 null */; nickname: string
  nicknameChangeableAt: string | null  // 이 시각 전에는 닉네임 변경 불가(null = 지금 가능)
  hasPassword: boolean                 // false(카카오)면 비밀번호 변경 메뉴 숨김
  balance: number                      // 헤더 잔액 알약
  liveBroadcastId: number | null       // 내가 지금 LIVE 중인 방송
}

type Category = 'just_chatting' | 'game' | 'fps' | 'rpg' | 'horror' | 'virtual' | 'sports' | 'music'
// 화면 라벨: 저스트채팅·종합게임·FPS·RPG·공포게임·버추얼·스포츠·음악 (프론트 상수)
type PipCorner = 'top_left' | 'top_right' | 'bottom_left' | 'bottom_right'
type BroadcastStatus = 'READY' | 'LIVE' | 'ENDED'

type Broadcast = {
  id: number; owner: UserSummary; status: BroadcastStatus
  title: string; category: Category; tags: string[]; record: boolean; pipCorner: PipCorner
  thumbnailUrl: string | null          // `/media/thumbs/{id}.jpg?v=<갱신시각>`, null이면 기본 이미지
  viewerCount: number                  // LIVE만 의미, 그 외 0
  peakViewers: number
  startedAt: string | null; endedAt: string | null; createdAt: string
  reconnecting: boolean                // 송출자 끊김 60초 유예 중
  superClutchUntil: string | null      // SUPER CLUTCH 진행 중이면 종료 시각(배지)
  recording: { id: number; status: RecordingStatus } | null   // record=false거나 아직 없음 → null
}

type BroadcastDetail = Broadcast & {
  gauge: ClutchGauge                   // LIVE가 아니면 percent 0, phase 'charging'
  burstPrice: number                   // SUPER BURST 가격(설정값)
  viewer: {                            // 요청자 기준. 비로그인은 null
    isOwner: boolean
    ban: { permanent: boolean; until: string | null } | null   // 이 스트리머에게 차단된 상태(풀린 임시차단은 null)
    freeClutchesLeft: number           // 30초 창 안에 남은 무료 클러치(0~5)
  } | null
}

type BroadcastRef = { id: number; title: string; hidden: boolean }  // 지갑 내역용. hidden=true면 "삭제된 방송"

type Chat = {
  id: number; broadcastId: number; author: UserSummary
  body: string | null                  // 욕설은 *** 로 가린 결과. 삭제됐으면 null
  edited: boolean; deleted: boolean    // "(수정됨)" / "삭제된 메시지입니다"
  badges: ('streamer' | 'donor')[]     // donor = 이 방송에 donation·burst 후원 이력(익명 포함)
  createdAt: string
}

type Donation = {                      // 공개형 / 소유자형 두 가지
  id: number; broadcastId: number; kind: 'donation' | 'burst'
  amount: number; message: string | null   // 욕설 가림 적용
  anonymous: boolean
  donor: UserSummary | null            // 공개형: anonymous면 null("익명의 후원자"로 표시). 소유자형: 항상 실제 사용자
  createdAt: string
}

type ClutchGauge = {
  percent: number                      // 0~100 정수
  phase: 'charging' | 'imminent' | 'cooldown'   // imminent = 80% 이상(설정값)
  cooldownUntil: string | null         // phase=cooldown일 때 재발동 가능 시각
}

type RecordingStatus = 'recording' | 'ready' | 'failed' | 'deleted'
type RecordingCard = {
  id: number; broadcastId: number; owner: UserSummary
  title: string; category: Category; thumbnailUrl: string | null
  durationSec: number; peakViewers: number; views: number; startedAt: string
}
type RecordingDetail = Omit<RecordingCard, 'durationSec'> & {
  status: RecordingStatus
  durationSec: number | null
  videoUrl: string | null              // status=ready만. Object Storage 공개 URL
  expiresAt: string | null
}

type LedgerEntry = {
  id: number                           // 묶음이면 묶음의 마지막 원장 id
  type: 'charge' | 'donation' | 'clutch' | 'burst' | 'withdraw_forfeit'
  amount: number                       // charge만 +, 나머지 -
  balanceAfter: number
  count: number                        // clutch 묶음의 건수, 그 외 1
  broadcast: BroadcastRef | null
  createdAt: string
}

type LiveKitToken = { token: string; url: string /* wss://… */; identity: string; room: string }
type Page<T> = { items: T[]; nextCursor: string | null }
```

## 2. 계정·인증 (f01)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| POST `/auth/signup` | 누구나 | `{loginId, password, nickname, agreeTerms: true, agreePrivacy: true, over14: true}` | 201 `Me` + 세션 쿠키(7일) | 422 `VALIDATION_FAILED`, 409 `AUTH_LOGIN_ID_TAKEN`·`AUTH_NICKNAME_TAKEN`, 429 `RATE_LIMITED`(IP 1시간 5회) |
| POST `/auth/login` | 누구나 | `{loginId, password, remember: boolean}` | 200 `Me` + 쿠키(7일, remember면 30일) | 401 `AUTH_INVALID_CREDENTIALS`, 429 `AUTH_LOCKED`(+`retryAfterSec`)·`RATE_LIMITED`(IP 1분 20회) |
| POST `/auth/test-login` | 누구나 | `{account: 'streamer1' \| 'streamer2' \| 'viewer'}` | 200 `Me` + 쿠키 | 422, 429 `RATE_LIMITED`(로그인과 같은 IP 한도) |
| POST `/auth/logout` | 누구나 | — | 204, 세션 삭제·쿠키 만료(세션 없어도 204) | — |
| GET `/me` | 로그인 | — | 200 `Me` + 쿠키 만료 갱신(7일/30일) | 401 |
| PATCH `/me` | 로그인 | `{nickname}` | 200 `Me` (지금과 같은 닉네임이면 변경 없이 200) | 401, 403 `AUTH_SEED_ACCOUNT_LOCKED`(테스트 계정), 422, 409 `AUTH_NICKNAME_TAKEN`·`AUTH_NICKNAME_CHANGE_TOO_SOON` |
| PUT `/me/password` | 로그인·비밀번호 계정 | `{currentPassword, newPassword}` | 204, **현재 세션 외 전부 삭제** | 401, 403 `FORBIDDEN`(카카오 계정), 403 `AUTH_SEED_ACCOUNT_LOCKED`(테스트 계정), 422 `VALIDATION_FAILED`·`AUTH_WRONG_PASSWORD` |
| DELETE `/me` | 로그인 | `{password, confirmForfeit: boolean}` | 204, 세션 전부 삭제 | 401, 422 `AUTH_WRONG_PASSWORD`, 409 `AUTH_WITHDRAW_WHILE_LIVE`·`AUTH_WITHDRAW_CONFIRM_FORFEIT` |

검증 규칙 (실패 시 `fields.<이름>`에 이유):
- `loginId`: `^[a-z0-9]{4,20}$`(서버가 소문자로 바꾼 뒤 검사). 탈퇴한 아이디 포함 영구 중복 불가 → `AUTH_LOGIN_ID_TAKEN`.
- `password`/`newPassword`: 8~64자, 영문·숫자 각 1자 이상, 아이디와 같으면 안 됨, 흔한 비밀번호 상위 100개 금지.
- `nickname`: 2~12자 `^[가-힣a-zA-Z0-9]+$`, 예약어·욕설·FLUTCH 금지어 불가(422). 활성 사용자 또는 **탈퇴 30일 이내** 사용자와 대소문자 무시 중복이면 `AUTH_NICKNAME_TAKEN`.
- `agreeTerms`·`agreePrivacy`·`over14`가 `true`가 아니면 422.
- 로그인: 같은 아이디 5회 연속 실패 → 5분 잠금(`AUTH_LOCKED`, 5번째 실패 응답부터. 잠금 중엔 비밀번호가 맞아도 거절, 성공하면 실패 횟수 0). 없는 아이디·틀린 비밀번호·탈퇴 계정 모두 `AUTH_INVALID_CREDENTIALS`.
- 요청 제한 IP는 `X-Forwarded-For` 첫 값(nginx가 `$remote_addr`로 덮어씀). 가입 제한은 검증(422)을 통과한 시도만 센다.
- 세션: 7일(사용 시 연장) / remember 30일. LIVE 송출 중인 사용자의 세션은 만료되지 않는다. 쿠키 `HttpOnly; Secure; SameSite=Lax; Path=/`(로컬 `http://localhost`도 브라우저가 Secure 쿠키를 받는다).
- 닉네임 변경: `nicknameChangedAt + 30일` 전이면 `AUTH_NICKNAME_CHANGE_TOO_SOON`(message에 남은 일수). 가입 시 설정은 기록하지 않는다.
- 탈퇴: 내 방송이 LIVE면 `AUTH_WITHDRAW_WHILE_LIVE`. 잔액 > 0이고 `confirmForfeit=false`면 `AUTH_WITHDRAW_CONFIRM_FORFEIT`(message "남은 N비트는 사라지고 되돌릴 수 없어요") → 확인 후 원장 `withdraw_forfeit -N` 추가하고 탈퇴. 카카오 계정은 `password` 없이 `confirmForfeit`만(T28).
- 테스트 로그인: 시드 계정 3개(서버 시작 시 없으면 생성, 비밀번호 `SEED_PASSWORD`)만. 다른 값은 422 `fields.account`. 쿠키는 7일.
- 시나리오: `auth_signup_login_logout` `auth_lockout` `auth_password_change_revokes_other_sessions` `auth_nickname_change_30day_limit` `auth_withdraw_forfeits_balance`

**카카오 로그인 (T28, 윤곽만 — T28에서 이 절 확정)**: GET `/auth/kakao/start` → 302 카카오(`state` 쿠키) · GET `/auth/kakao/callback?code&state` → 기존 계정이면 세션 발급 후 302 `/`, 처음이면 302 `/signup/kakao`(임시 서명 쿠키) · POST `/auth/kakao/signup {nickname, agreeTerms, agreePrivacy, over14}` → 201 `Me`.

## 3. 방송 (f02, f03)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| GET `/broadcasts` | 누구나 | `?category=&q=&cursor=&limit=` | 200 `Page<Broadcast>` — **LIVE·미숨김만**, 시청자 수 내림차순 | 422 |
| POST `/broadcasts` | 로그인 | `{title, category, tags?: string[], record?: boolean(기본 true)}` | 201 `BroadcastDetail` (READY) | 401, 422, 409 `BROADCAST_READY_LIMIT` |
| GET `/broadcasts/{id}` | 누구나(READY도 링크로 열림) | — | 200 `BroadcastDetail` | 404(없음·숨김) |
| PATCH `/broadcasts/{id}` | 소유자 | `{title?, category?, tags?, record?, pipCorner?}` | 200 `BroadcastDetail`, `broadcast.updated` 발행 | 401, 404, 403 `BROADCAST_NOT_OWNER`, 422, 409 `BROADCAST_FIELD_LOCKED` |
| DELETE `/broadcasts/{id}` | 소유자 | — | 204 | 401, 404, 403 `BROADCAST_NOT_OWNER` |
| PUT `/broadcasts/{id}/thumbnail` | 소유자·LIVE | 본문 이미지 바이트, `Content-Type: image/jpeg`, 최대 512KB | 204 (최신 1장만 덮어씀, `thumbnailUrl`의 `?v=`가 바뀌고 `broadcast.updated` 발행) | 401, 404, 403, 422 `BROADCAST_THUMBNAIL_INVALID`, 409 `BROADCAST_NOT_LIVE` |
| GET `/me/broadcasts` | 로그인 | `?status=READY\|LIVE\|ENDED&cursor=&limit=` | 200 `Page<Broadcast>` — 내 방송(미숨김) 최신 등록순, READY 포함 | 401 |
| GET `/users/{id}/broadcasts` | 누구나 | `?cursor=&limit=` | 200 `Page<Broadcast>` — 그 스트리머의 **ENDED·미숨김** 최신순(다시보기는 `recording`) | 404 |

- 검증: `title` 2~50자·FLUTCH 금지어/욕설 불가, `category`는 8개 코드 중 하나, `tags` 최대 3개·각 1~10자·금지어 불가·중복 제거, `pipCorner`는 4개 값 중 하나. 실패는 422 `fields`.
- `q`: 1~30자, 제목·닉네임·태그 부분 일치(대소문자 무시). `category`와 함께 쓸 수 있다. 다시보기 검색은 `GET /recordings?q=`(검색 화면은 LIVE 결과 뒤에 다시보기 결과를 이어 붙인다).
- 상태별 수정 가능 필드 (그 외 필드가 본문에 있으면 409 `BROADCAST_FIELD_LOCKED`):

  | 상태 | 수정 가능 |
  |---|---|
  | READY | `title` `category` `tags` `record` `pipCorner` |
  | LIVE | `title` `category` `tags` `pipCorner` |
  | ENDED | `title` |
- 목록 정렬·커서: `GET /me/broadcasts`·`GET /users/{id}/broadcasts`는 최신 id 순이고 `nextCursor`는 마지막 항목 id다(T04). `GET /broadcasts`는 **시청자 수 내림차순, 동률은 최신순**이고 `nextCursor`는 순위 위치(앞에서 건너뛸 개수)다 — 시청자 수가 서버 메모리 값이라 조건에 맞는 LIVE를 전부 읽어 정렬한다(T08b). 페이지 사이에 시청자 수가 바뀌면 경계에서 한 방송이 겹치거나 빠질 수 있다(홈은 10초 폴링). 잘못된 `limit`(숫자 아님)·`cursor`·`category`·`status`는 422, `limit`은 1~50으로 보정한다.
- 등록 한도: 내 READY가 이미 5개면 `BROADCAST_READY_LIMIT`. "이전 방송 정보 복사"는 별도 API 없이 프론트가 `GET /me/broadcasts`의 값을 등록 폼에 채운다.
- 삭제: READY → 행 삭제. LIVE → 송출 종료(§3-1 종료 절차, `broadcast.ended` reason `deleted`) 후 ENDED 규칙. ENDED → `hiddenAt` 기록(이후 모든 조회·하위 리소스·WebSocket 404) + 녹화 파일 삭제(`recordings.status=deleted`). 채팅·후원·원장은 보존.
- 방송 길이: LIVE 3시간(설정값 `LIVE_MAX_DURATION_SECS`)에 자동 종료(reason `max_duration`), 10분 전 `broadcast.duration_warning` 한 번(서버 메모리로 중복 방지 — 경고 구간에 서버가 재시작하면 한 번 더 갈 수 있다). 판정은 5초 주기 작업이라 최대 5초 늦다.
- 시나리오: `broadcast_owner_crud` `broadcast_other_user_forbidden` `broadcast_public_read` `broadcast_state_transitions` `broadcast_limits` `broadcast_delete_live_ends_first` `broadcast_persists_after_restart` `broadcast_thumbnail_owner_live_only`

### 3-1. 송출·시청 토큰 (f03)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| POST `/broadcasts/{id}/live` | 소유자 | `{tabId, takeover?: boolean}` | 200 `{broadcast: BroadcastDetail, livekit: LiveKitToken}` (publish 토큰) | 401, 404, 403 `BROADCAST_NOT_OWNER`, 422, 409 `BROADCAST_INVALID_TRANSITION`·`BROADCAST_LIVE_LIMIT`·`LIVE_PUBLISHER_ACTIVE` |
| DELETE `/broadcasts/{id}/live` | 소유자 | — | 200 `BroadcastDetail` (ENDED) | 401, 404, 403, 409 `BROADCAST_INVALID_TRANSITION` |
| POST `/broadcasts/{id}/viewer-token` | 누구나 | `{tabId, guestId?}` (비로그인은 `guestId` 필수) | 200 `LiveKitToken` (subscribe 전용) | 404, 422, 409 `BROADCAST_NOT_LIVE` |

`POST /live`는 "시작 또는 이어서 송출"이다:
| 현재 상태 | 동작 |
|---|---|
| READY | 내 다른 방송이 LIVE면 `BROADCAST_LIVE_LIMIT`. 아니면 LIVE 전이(`startedAt`) + 토큰 + 녹화 켜짐이면 Egress 시작 + `broadcast.updated`. 동시에 온 다른 시작 요청이 먼저 LIVE로 바꿨으면 409 대신 아래 LIVE 규칙으로 판단(T15a) |
| LIVE, 송출자 없음(유예 중 `reconnecting=true`) 또는 같은 `identity` | 새 토큰만 발급(복귀). 참가 웹훅이 오면 `disconnectedAt` 해제 + `broadcast.resumed` |
| LIVE, 다른 기기(`identity` 다름)가 송출 중 | `takeover`가 아니면 409 `LIVE_PUBLISHER_ACTIVE` → `takeover: true`면 기존 참가자 제거 후 새 토큰, `publisherIdentity` 교체 |
| ENDED | 409 `BROADCAST_INVALID_TRANSITION` |

- "송출 중" 판정은 DB만 본다(T08a): `publisherIdentity`가 있고 요청 신원과 다르며 `disconnectedAt`이 없으면 다른 기기가 송출 중. LiveKit 실제 접속은 조회하지 않으므로 퇴장 웹훅이 `disconnectedAt`을 기록하기 전(탭을 닫은 직후 몇 초)에는 새 탭에서 시작해도 전환 확인(409)이 뜬다. 신원이 바뀌면(전환·유예 중 다른 탭 복귀) 이전 신원을 LiveKit에서 내보낸다(이미 없으면 무시). 밀려난 기기가 옛 송출 토큰(TTL 4시간)으로 다시 접속하면 참가 웹훅에서 `canPublish`인 `u:` 참가자 중 `publisherIdentity`가 아니거나 방송이 LIVE가 아닌 참가자를 내보낸다(끝난 방송의 방이 다시 살아나지 않게, T15a).
- **송출자 미접속**(T15a): 송출자 신원이 처음 정해지거나 바뀔 때(`POST /live`) `disconnectedAt`도 함께 기록한다(이미 있으면 유지). 그 신원의 참가 웹훅이 오면 해제(+ `broadcast.resumed`)되고, 끝내 접속하지 못하면(연결 실패·탭 닫음 — 웹훅 없음) 60초 유예 만료로 종료된다. 그래서 `POST /live` 응답과 접속 전 몇 초 동안 `reconnecting=true`로 보인다(`broadcast.reconnecting` 이벤트는 보내지 않는다). 접속 전인 기기는 "송출 중"이 아니므로 그 사이 다른 기기의 시작은 확인 없이 전환된다.
- **60초 유예**(설정값 `LIVE_GRACE_SECS`): 송출자 퇴장 웹훅 → `disconnectedAt` 기록 + `broadcast.reconnecting`. 송출자 신원의 참가 웹훅 → 해제 + `broadcast.resumed`. **송출자 판정은 신원 + `canPublish`**: 같은 신원이라도 구독 전용 참가자(같은 탭의 시청 토큰)는 유예를 풀거나 시작하지 않는다(T15a). 5초 주기 작업이 유예가 지난 LIVE를 찾으면 LiveKit 참가자 목록을 한 번 확인해(웹훅 순서가 뒤바뀌는 경우) 송출 권한의 송출자가 있으면 해제, 없으면(또는 확인 실패) 종료(reason `disconnect`). 종료 UPDATE는 `disconnectedAt`이 남아 있을 때만 — 확인과 종료 사이에 참가 웹훅이 유예를 풀었으면 끝내지 않는다.
- **서버 재시작**: 시작 후 첫 주기에 LIVE 방송마다 LiveKit 참가자 목록으로 시청자 수를 복구하고, 송출자가 없으면 유예 시작(60초 뒤 자연 종료), 유예 중인데 송출자가 있으면 해제. LiveKit 호출이 실패하면 DB를 바꾸지 않고 다음 주기에 다시 한다.
- TS 타입: 요청 `StartLiveRequest`·`ViewerTokenRequest`, 응답 `StartLiveResponse`·`LiveKitToken`. 형식 오류는 422 `fields.tabId`·`fields.guestId`.
- 종료(`DELETE /live`, 60초 유예 만료, 3시간 초과, LIVE 중 삭제 공통): ENDED 전이(`endedAt`) → LiveKit 방 삭제(송출 실제 중단) → Egress 종료 → `broadcast.ended`. READY·ENDED에서 `DELETE /live`는 `BROADCAST_INVALID_TRANSITION`.
- **LiveKit 신원**: 로그인 `u:<userId>:<tabId>`, 비로그인 `g:<guestId>:<tabId>`. 방 이름 `broadcast-<id>`. `tabId`는 탭마다 `sessionStorage`의 랜덤값, `guestId`는 브라우저 `localStorage`의 랜덤값, 둘 다 `^[A-Za-z0-9_-]{8,64}$`(아니면 422). 로그인 사용자의 `guestId`는 무시한다.
- 권한: publish 토큰 = `roomJoin, canPublish(camera·microphone·screen_share·screen_share_audio), canSubscribe`, 시청 토큰 = `roomJoin, canSubscribe, hidden`(`canPublish=false` 명시 — LiveKit은 빠지면 true로 본다). 둘 다 `canPublishData=false`, TTL 4시간. `hidden`이면 다른 참가자(송출자·시청자 클라이언트)에게 시청자 신원이 보이지 않는다. 참가·퇴장 웹훅과 서버 API 참가자 목록에는 그대로 나온다(로컬 LiveKit v1.13 실측, T15a).
- 시청자 수: 참가·퇴장 웹훅으로 서버 메모리에 방송별 신원 → 접속 `sid`를 두고(같은 신원 재접속은 새 참가가 옛 퇴장보다 먼저 올 수 있어 퇴장은 `sid`가 같을 때만 지운다), 사용자 단위(`u:<id>`/`g:<id>`)로 묶어 센다(같은 사용자 여러 탭은 1명). **방송 소유자(`u:<ownerId>:` 전부)와 우리 형식이 아닌 신원(Egress 등)은 제외**. 바뀔 때 `viewer.count`, `peakViewers`는 올라갈 때만 DB에 저장.
- READY 방송 시청 화면은 토큰 없이 대기하고, WebSocket `broadcast.updated`(status LIVE)를 받으면 시청 토큰을 요청한다.
- 시나리오: `live_start_owner_only` `live_end_closes_room` `live_viewer_token_rules` `live_disconnect_grace_auto_end` `live_takeover_other_device` `live_max_duration_auto_end` `live_webhook_signature_invalid_rejected` `live_viewer_count` `live_restart_sync` `live_publisher_never_joins_auto_end` `live_subscriber_same_identity_keeps_grace`

## 4. 채팅 (f04)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| GET `/broadcasts/{id}/chats` | 누구나(모든 상태) | `?before=<chatId>&limit=<기본 50, 최대 100>` | 200 `{items: Chat[] /* 오래된→최신 */, hasMore: boolean}` | 404 |
| POST `/broadcasts/{id}/chats` | 로그인·미차단·LIVE | `{body}` | 201 `Chat`, `chat.created` | 401, 404, 403 `MODERATION_TIMED_OUT`·`MODERATION_BANNED`, 422 `VALIDATION_FAILED`·`CHAT_BANNED_WORD`, 409 `BROADCAST_NOT_LIVE`, 429 `RATE_LIMITED`·`CHAT_REPEATED` |
| PATCH `/chats/{id}` | 작성자·미차단·LIVE·5분 이내 | `{body}` | 200 `Chat`, `chat.updated` | 401, 404, 403 `CHAT_NOT_AUTHOR`·`MODERATION_*`, 422, 409 `BROADCAST_NOT_LIVE`·`CHAT_EDIT_EXPIRED`·`CHAT_DELETED` |
| DELETE `/chats/{id}` | 작성자 또는 방송 소유자, 언제든 | — | 204, `chat.deleted` (이미 삭제면 204) | 401, 404, 403 `CHAT_DELETE_FORBIDDEN` |

- `body` 1~200자. 1초 1개 초과 → `RATE_LIMITED`. 같은 사용자·같은 방송에서 같은 본문 30초 안 3번째 → `CHAT_REPEATED`.
- 욕설은 원문 저장, **응답·이벤트에는 `***`로 가린 결과만**. FLUTCH 금지어 포함은 `CHAT_BANNED_WORD`로 저장 안 함. 수정에도 같은 검사.
- 입장 시 `GET …/chats`(before 없음) = 최근 50개, 위로 스크롤 시 `before=items[0].id`. ENDED·다시보기도 같은 API.
- 채팅 기록에는 채팅만 있다. 후원 강조 줄·시스템 메시지(차단 알림)는 실시간 이벤트로만 그리고 저장하지 않는다.
- 시나리오: `chat_live_only` `chat_guest_read_only` `chat_edit_author_only_within_5min` `chat_delete_author_or_owner` `chat_rate_limit_and_repeat` `chat_profanity_masked_banned_word_blocked`

## 5. 모더레이션 (f05)
경로의 `{id}`는 **스트리머 id**이고 로그인 사용자 본인이어야 한다(아니면 403 `MODERATION_NOT_OWNER`).

| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| GET `/users/{id}/bans` | 본인 | `?cursor=&limit=` | 200 `Page<Ban>` — 현재 유효한 차단만(풀린 임시차단 제외), 최신순 | 401, 403 |
| PUT `/users/{id}/bans/{userId}` | 본인 | `{duration: '1m' \| '10m' \| '1h' \| 'permanent'}` | 200 `Ban` (기존 차단은 덮어씀) | 401, 403, 404(대상 없음), 422 `MODERATION_SELF`·`VALIDATION_FAILED` |
| DELETE `/users/{id}/bans/{userId}` | 본인 | — | 204 (차단 없어도 204) | 401, 403 |
| GET `/users/{id}/viewers/{userId}/chats` | 본인 | `?before=&limit=` | 200 `{items: Chat[], hasMore}` — 그 시청자가 **내 방송들**에서 쓴 채팅 | 401, 403, 404 |

```ts
type Ban = { user: UserSummary; permanent: boolean; until: string | null; createdAt: string }
```
- 차단·해제마다 `ban_events`에 추가(추가만). 해제는 `bans` 행 삭제.
- 차단 시 그 스트리머의 LIVE 방송이 있으면 `chat.system` 발행. **영구 밴이면 그 방송의 대상자 채팅을 전부 삭제 처리**(`deletedAt`)하고, 클라이언트는 `chat.system`(kind `ban`)을 받으면 그 사용자의 메시지를 "삭제된 메시지"로 바꾼다.
- 차단 효과는 각 엔드포인트가 강제한다:

  | 동작 (엔드포인트) | 임시차단 | 영구 밴 |
  |---|---|---|
  | 시청 (`viewer-token`, 조회) | 가능 | 가능 |
  | 채팅 작성·수정 (§4) | 403 `MODERATION_TIMED_OUT` | 403 `MODERATION_BANNED` |
  | 무료 클러치 (§7) | 가능 | 가능 |
  | 유료 클러치·SUPER BURST (§7) | 가능 | 403 `MODERATION_BANNED` |
  | 후원·후원 주문 (§6) | 가능 | 403 `MODERATION_BANNED` |
- 입력창 표시는 `BroadcastDetail.viewer.ban`으로("n분 후 채팅할 수 있어요" / "이 채널에서 채팅이 제한되었어요").
- 시나리오: `ban_blocks_chat_across_broadcasts` `timeout_expires_automatically` `ban_effects_matrix` `ban_only_by_owner` `permanent_ban_hides_current_messages`

## 6. 지갑·결제·후원 (f06)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| GET `/me/wallet` | 로그인 | `?type=&month=YYYY-MM&cursor=&limit=` | 200 `{balance, items: LedgerEntry[], nextCursor}` 최신순 | 401, 422 |
| POST `/payments/orders` | 로그인 | 충전 `{purpose: 'charge', amount}` / 후원 `{purpose: 'donation', broadcastId, donation: {amount, message?, anonymous}}` | 201 `PaymentOrder` | 401, 404, 403 `MODERATION_BANNED`·`DONATION_SELF`, 422, 409 `WALLET_BALANCE_LIMIT`·`BROADCAST_NOT_LIVE`·`PAYMENT_NOT_NEEDED` |
| POST `/payments/confirm` | 로그인·주문자 | `{paymentKey, orderId, amount}` (토스 성공 리다이렉트 쿼리 그대로) | 200 `PaymentResult` | 401, 404 `PAYMENT_ORDER_NOT_FOUND`, 403 `PAYMENT_NOT_OWNER`, 409 `PAYMENT_AMOUNT_MISMATCH`·`PAYMENT_ORDER_EXPIRED`·`PAYMENT_ALREADY_CONFIRMED`·`PAYMENT_APPROVAL_FAILED`·`WALLET_BALANCE_LIMIT`, 502 `PAYMENT_PROVIDER_UNAVAILABLE` |
| POST `/payments/webhook` | 토스(Origin 예외) | 토스 `PAYMENT_STATUS_CHANGED` 본문 | 200 (처리·무시 모두) | 500만(토스가 재시도) |
| POST `/broadcasts/{id}/donations` | 로그인·LIVE·미밴·소유자 아님 | `{amount, message?, anonymous}` | 201 `{donation: Donation /* 내 후원이라 실명 */, balance}`, `donation.created` | 401, 404, 403 `MODERATION_BANNED`·`DONATION_SELF`, 422 `VALIDATION_FAILED`·`DONATION_BANNED_WORD`, 409 `BROADCAST_NOT_LIVE`·`WALLET_INSUFFICIENT_BALANCE` |
| GET `/broadcasts/{id}/donations` | 누구나(소유자는 실명) | `?cursor=&limit=` | 200 `DonationList` | 404 |
| POST `/donations/{id}/skip` | 그 방송 소유자 | — | 204, `donation.skipped` | 401, 404, 403 `BROADCAST_NOT_OWNER` |
| GET `/me/donation-stats` | 로그인 | `?cursor=&limit=` | 200 `DonationStats` | 401 |

```ts
type PaymentOrder = { orderId: string; amount: number /* 결제할 금액 */; orderName: string /* "1,000비트 충전" */; expiresAt: string }
type PaymentResult = {
  orderId: string; purpose: 'charge' | 'donation'; charged: number; balance: number
  donation: Donation | null            // 후원 주문이 후원까지 확정됐을 때(내 후원이라 실명)
  notice: string | null                // 예: "방송이 종료되어 잔액으로 충전했어요"
}
type DonationList = Page<Donation> & {
  total: number                        // 이 방송이 받은 비트 합계(donation + clutch + burst)
  count: number                        // donation·burst 건수
  clutchTotal: number                  // 유료 클러치 합계(목록에는 건별로 나오지 않음)
}
type DonationStats = Page<{ broadcast: BroadcastRef & { startedAt: string }; total: number; count: number }> & {
  totalReceived: number                // 누적 받은 비트
  last7Days: number                    // 최근 7일(KST) 합계
}
```

- 금액: 충전 `amount`·후원 `amount` 모두 1,000~1,000,000 정수. 충전 후 잔액이 3,000,000을 넘으면 `WALLET_BALANCE_LIMIT`. 후원 `message` 0~100자, 욕설 가림·FLUTCH 금지어 `DONATION_BANNED_WORD`.
- **후원 주문의 결제 금액은 서버가 계산**: N = max(후원액 − 현재 잔액, 1,000). 잔액이 이미 충분하면 `PAYMENT_NOT_NEEDED`(잔액 후원을 쓰라는 뜻). 클라이언트가 보낸 결제 금액은 받지 않는다.
- 주문은 30분 후 만료(`PAYMENT_ORDER_EXPIRED`). `orderId`는 서버 생성 `^[A-Za-z0-9_-]{6,64}$`. 토스 위젯 `customerKey`는 `ANONYMOUS`.
- confirm 순서: 주문 조회·주문자 확인 → `amount`가 주문 금액과 다르면 `PAYMENT_AMOUNT_MISMATCH`(토스 호출 안 함) → 만료 확인 → 토스 승인 API → **한 트랜잭션**: 주문 `done` + 원장 `charge +N` + (후원 주문이고 방송이 LIVE·미밴이면) 원장 `donation −금액` + 후원 기록 + 게이지 점수. 방송이 끝났거나 그 사이 밴되면 충전만 하고 `notice`.
- **멱등**: 이미 `done`인 주문에 같은 `paymentKey`로 다시 confirm → 200 같은 `PaymentResult`(재적립 없음). 다른 `paymentKey` → `PAYMENT_ALREADY_CONFIRMED`. 웹훅은 본문 `paymentKey`로 토스 결제 조회 API를 다시 불러 상태·금액·`secret`을 확인한 뒤 같은 확정 함수를 탄다(`payment_key`·`ledger_entries.payment_order_id` 유니크).
- 잔액 후원: `UPDATE wallets … WHERE balance >= $amount` 조건부 차감, 0행이면 `WALLET_INSUFFICIENT_BALANCE` → 화면은 "N비트 충전하고 후원하기"(후원 주문)로 전환.
- 지갑 `type` 필터는 원장 유형 5개 중 하나, `month`는 KST 기준 월. **유료 클러치는 같은 방송의 연속된 원장을 한 줄로 묶어**(`count`, 합계 `amount`) 내려준다. 숨긴 방송은 `broadcast.hidden=true`.
- 후원 목록: kind `donation`·`burst`만 최신순. 공개 응답은 익명 후원의 `donor=null`, **방송 소유자 요청에만 실제 `donor`**. 숨긴 방송이면 404.
- 스킵: 소유자가 현재 표시 중인 알림을 넘김 → 모든 연결에 `donation.skipped`.
- 시나리오: `donation_confirm_idempotent` `donation_amount_mismatch_rejected` `donation_balance_conditional_debit` `donation_live_only` `donation_after_end_becomes_charge` `donation_anonymous_name_visibility` `charge_limits_and_order_expiry` `banned_user_cannot_donate`

## 7. Clutch (f07)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| POST `/broadcasts/{id}/clutches` | 로그인·LIVE·소유자 아님 | — | 200 `{paid: boolean, combo: number /* 무료 1~5, 유료 0 */, freeClutchesLeft, balance: number \| null /* 유료일 때만 */, gauge: ClutchGauge}` | 401, 404, 403 `CLUTCH_SELF`·`MODERATION_BANNED`(유료만), 409 `BROADCAST_NOT_LIVE`·`WALLET_INSUFFICIENT_BALANCE`, 429 `RATE_LIMITED`(초당 10회) |
| POST `/broadcasts/{id}/bursts` | 로그인·LIVE·미밴·소유자 아님 | `{anonymous: boolean}` | 200 `{donation: Donation, balance, gauge}`, `donation.created` | 401, 404, 403 `CLUTCH_SELF`·`MODERATION_BANNED`, 409 `BROADCAST_NOT_LIVE`·`WALLET_INSUFFICIENT_BALANCE` |
| GET `/home/hero` | 누구나 | — | 200 `Hero` | — |
| GET `/clips` (T30) | 누구나 | `?cursor=&limit=` | 200 `Page<Clip>` 최신순 | — |

```ts
type Hero = {
  mode: 'super_clutch' | 'top_gauge' | 'top_viewers' | 'empty'
  broadcast: Broadcast | null          // 지금 히어로 방송(empty면 null)
  gauge: ClutchGauge | null
  slot: { startedAt: string; nextSwitchAt: string } | null   // super_clutch 회전 중일 때 30초 슬롯
  active: { broadcast: Broadcast; expiresAt: string }[]       // 진행 중 SUPER CLUTCH, 최대 5, 회전 순서
}
type Clip = { id: number /* super_clutch id */; recordingId: number; videoUrl: string; broadcast: Broadcast; startSec: number; durationSec: number; triggeredAt: string }
```
- 클러치: 사용자·방송별 **최근 30초 무료 5회**, 6번째부터 1회 1비트 조건부 차감(원장 `clutch`, 후원 기록 kind `clutch`, 알림 없음). 점수: 클러치 50점, 후원·버스트 1비트당 10점(설정값).
- 버스트: 가격 `burstPrice`(설정값 1,000) 차감, 원장 `burst` + 후원 기록 kind `burst`(표시 이름은 `anonymous`).
- 게이지는 최근 5분 슬라이딩 합계 / 10,000점. 100% 도달 시 쿨다운(진행 중이거나 마지막 발동 30분 이내)이 아니면 SUPER CLUTCH: 게이지 0, `super_clutches` 추가(5분, 회전 바로 다음 순서), `clutch.super` 발행. 쿨다운 중이면 게이지만 오르고 phase `cooldown`.
- 히어로: 진행 중 SUPER CLUTCH가 있으면 30초씩 회전(`mode=super_clutch`, 서버 1초 스케줄러가 DB 기준으로 진행), 없으면 게이지 1위 LIVE(`top_gauge`), 모두 0이면 시청자 수 1위(`top_viewers`), LIVE가 없으면 `empty`. 홈은 10초 폴링하고 `slot.nextSwitchAt`이 더 이르면 그때 다시 조회.
- 클립: 녹화 `ready`인 방송의 SUPER CLUTCH마다 `startSec = max(0, 발동시각 − startedAt − 30)`, `durationSec = 45`. 클립이 없으면 홈은 `GET /recordings`(최근 다시보기)를 보여준다.
- 시나리오: `clutch_free_combo_then_paid` `clutch_gauge_decay_window` `super_clutch_trigger_reset_and_cooldown` `clutch_rotation` `clutch_banned_rules`

## 8. 녹화·다시보기 (f08)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| GET `/recordings` | 누구나 | `?category=&q=&userId=&cursor=&limit=` | 200 `Page<RecordingCard>` — `ready`·미숨김만, 최신순 | 422 |
| GET `/recordings/{id}` | 누구나 | — | 200 `RecordingDetail` (`recording`·`failed`도 200, `videoUrl=null`) | 404 `NOT_FOUND`(없음·숨김), 404 `RECORDING_EXPIRED`(deleted) |
| POST `/recordings/{id}/views` | 누구나 | `{guestId?}` (비로그인 필수) | 204 — 시청자(`u:<id>`/`g:<guestId>`)당 KST 하루 1회만 증가 | 404, 422 |

- 다시보기 화면의 채팅·후원은 `GET /broadcasts/{broadcastId}/chats`·`/donations`(전체 목록, 리플레이 없음).
- 녹화 실패는 방송 상태와 무관(방송은 정상 ENDED). 소유자 관리 화면은 `Broadcast.recording.status=failed`로 "녹화에 실패했어요".
- 시나리오: `recording_saved_after_end_survives_restart` `recording_failure_does_not_break_broadcast` `recording_deleted_with_broadcast_hide` `recording_retention_cleanup`

## 9. 팔로우·스트리머 페이지 (f09)
| 메서드·경로 | 권한 | 요청 | 응답 | 에러 |
|---|---|---|---|---|
| GET `/users/{id}` | 누구나 | — | 200 `{user: UserSummary, followerCount, following: boolean \| null /* 비로그인 null */, liveBroadcast: Broadcast \| null}` | 404(없음·탈퇴) |
| PUT `/users/{id}/follow` | 로그인 | — | 200 `{following: true, followerCount}` (이미 팔로우면 그대로) | 401, 404, 422 `FOLLOW_SELF` |
| DELETE `/users/{id}/follow` | 로그인 | — | 200 `{following: false, followerCount}` | 401, 404 |
| GET `/pulse` | 로그인 | `?limit=<기본 10, 최대 50>` | 200 `{live: Broadcast[] /* 시청자 수 순 */, offline: {user: UserSummary, followerCount, lastEndedAt: string \| null}[], total: number}` — live 먼저, 합쳐서 `limit`개 | 401 |

- 팔로워 목록 API는 없다(숫자만). 사이드바 "인기 라이브"는 `GET /broadcasts?limit=5`(팔로우 0개면 3), "추천 라이브 채널"은 `GET /broadcasts?limit=3`.
- 스트리머 페이지 관리 탭(본인만): 방송 관리 `GET /me/broadcasts`, 후원 내역·통계 `GET /me/donation-stats` + `GET /broadcasts/{id}/donations`, 차단 목록 `GET /users/{id}/bans`.
- 시나리오: `follow_toggle_and_count` `follow_self_rejected`

## 10. 웹훅·운영
| 메서드·경로 | 호출자 | 검증 | 처리 |
|---|---|---|---|
| POST `/livekit/webhook` | LiveKit (Origin 예외) | `Authorization` 헤더 JWT(API secret HS256, `iss`=API key, 만료 전) + `sha256` 클레임 = 본문 SHA-256의 표준 base64. 실패 401 `LIVE_WEBHOOK_INVALID_SIGNATURE`(WARN 로그). 본문(LiveKit `WebhookEvent` protojson)이 JSON이 아니면 422 | `participant_joined`/`participant_left`: 시청자 수 갱신(`sid` 비교), 송출 권한(`permission.canPublish`)의 송출자(`publisherIdentity`)면 유예 해제·시작, 송출 권한 `u:` 참가자인데 송출자가 아니거나 방송이 LIVE가 아니면 내보냄(§3-1). 웹훅 JSON은 camelCase이고 false는 생략된다(`canPublish`가 없으면 구독 전용) · `room_finished`: LIVE면 **유예 시작**(종료 아님 — LiveKit은 마지막 참가자가 나가고 약 20초 뒤 빈 방을 닫아 60초 유예보다 빠르다. 송출자가 돌아오면 방은 다시 생긴다) · `egress_ended`(T25, 지금은 무시): 녹화 `ready`(object_key·size·duration) 또는 `failed` + `recording.updated`. 그 외 이벤트는 200 무시 |
| POST `/payments/webhook` | 토스 (Origin 예외) | 서명 대신 결제 조회 API 재확인(§6) | 확정 또는 무시 |
| GET `/health` | 누구나 | — | 200 `{status: 'ok'}` / 503 `{status: 'unavailable'}` |
- 로컬 웹훅 주소 `http://host.docker.internal:<BACKEND_PORT>/api/v1/livekit/webhook`. `/metrics`는 API 밖(외부 비공개).

## 11. WebSocket `/api/v1/broadcasts/{id}/events`
- 업그레이드 시 `Origin` 검사(없거나 불일치 403, 방송 조회보다 먼저), 세션 쿠키가 있으면 사용자 식별(없으면 익명). 없는·숨긴 방송은 404. 소유자 여부는 연결 시점에 정한다. 클라이언트 → 서버 메시지는 없다(쓰기는 모두 REST, 받은 메시지는 무시). 서버가 30초마다 ping, 다음 ping까지 pong이 없으면 끊는다.
- 메시지 `{ "type": "chat.created", "data": {...} }`. **DB 커밋 후** 발행. 끊기면 자동 재접속 후 필요한 것을 다시 조회(이벤트 누락 허용).
- 대상: **public** = 모든 연결, **owner** = 방송 소유자 연결만, **split** = 소유자 연결엔 소유자형, 나머지엔 공개형(익명 후원자 `donor=null`). 이벤트를 만들 때 두 형태를 같이 만들어 연결 종류로 고른다.

| type | 대상 | data |
|---|---|---|
| `chat.created` / `chat.updated` | public | `Chat` |
| `chat.deleted` | public | `{id}` |
| `chat.system` | public | `{kind: 'timeout' \| 'ban', user: UserSummary, until: string \| null}` → "OOO님의 채팅이 10분간 제한되었어요" / "OOO님이 차단되었어요". `ban`이면 그 사용자 메시지 전부 삭제 표시 |
| `broadcast.updated` | public | `Broadcast` (제목·카테고리·태그·PIP·LIVE 전이·썸네일) |
| `broadcast.ended` | public | `{endedAt, peakViewers, durationSec, reason: 'owner' \| 'disconnect' \| 'max_duration' \| 'deleted', recording: {id, status} \| null}` |
| `broadcast.reconnecting` | public | `{since}` — 송출자 끊김, 60초 유예 시작 |
| `broadcast.resumed` | public | `{}` |
| `broadcast.duration_warning` | owner | `{endsAt}` — 자동 종료 10분 전 |
| `viewer.count` | public | `{count}` |
| `donation.created` | split | `{donation: Donation, total}` — kind `donation`·`burst`. 알림 10초, 여러 건이면 순서대로, 10,000비트 이상 큰 연출, 채팅창 강조 한 줄 |
| `donation.skipped` | public | `{id}` |
| `clutch.gauge` | public | `ClutchGauge` — 변경 시, 방송당 최대 초당 2회로 묶어 보냄 |
| `clutch.super` | public | `{triggeredAt, expiresAt}` — 3초 전체 화면 연출 |
| `recording.updated` | public | `{id, status}` — ENDED 화면 "다시보기 준비 중" → "다시보기 보기" |
| `resync` | 해당 연결 | `{}` — 서버 채널 지연(`Lagged`)으로 이벤트를 놓침. 상세·채팅을 다시 조회 |

- 화면 공유 중단·웹캠 켜짐 같은 트랙 변화는 시청자가 LiveKit 이벤트로 직접 감지한다.
- TS 타입: `WsEvent`(`type`으로 좁히는 유니온, 이벤트가 생길 때마다 변형이 늘어난다). 백엔드 발행은 `realtime::Event::{public, owner_only, split}` + `Hub::publish`.

## 12. 에러 코드
공통: `UNAUTHENTICATED`(401 "로그인이 필요합니다.") `FORBIDDEN`(403, Origin 포함) `NOT_FOUND`(404) `VALIDATION_FAILED`(422 + `fields`) `RATE_LIMITED`(429 + `retryAfterSec`) `CONFLICT`(409) `INTERNAL`(500).

| 코드 | 상태 | 사용자 문구 | 발생 |
|---|---|---|---|
| `AUTH_LOGIN_ID_TAKEN` | 409 | 이미 사용 중인 아이디예요. | signup |
| `AUTH_NICKNAME_TAKEN` | 409 | 이미 사용 중인 닉네임이에요. | signup, PATCH /me |
| `AUTH_INVALID_CREDENTIALS` | 401 | 아이디 또는 비밀번호가 올바르지 않아요. | login |
| `AUTH_LOCKED` | 429 | 로그인 시도가 많아 잠겼어요. N분 후 다시 시도해 주세요. (+`retryAfterSec`) | login |
| `AUTH_NICKNAME_CHANGE_TOO_SOON` | 409 | 닉네임은 N일 후에 바꿀 수 있어요. | PATCH /me |
| `AUTH_SEED_ACCOUNT_LOCKED` | 403 | 테스트 계정은 바꿀 수 없어요. | PATCH /me, PUT /me/password |
| `AUTH_WRONG_PASSWORD` | 422 | 현재 비밀번호가 올바르지 않아요. | PUT /me/password, DELETE /me |
| `AUTH_WITHDRAW_WHILE_LIVE` | 409 | 방송을 먼저 종료해 주세요. | DELETE /me |
| `AUTH_WITHDRAW_CONFIRM_FORFEIT` | 409 | 남은 N비트는 사라지고 되돌릴 수 없어요. | DELETE /me |
| `BROADCAST_NOT_OWNER` | 403 | 방송 소유자만 할 수 있어요. | 방송 수정·삭제·송출·썸네일·후원 스킵 |
| `BROADCAST_INVALID_TRANSITION` | 409 | 지금 방송 상태에서는 할 수 없어요. | POST·DELETE /live |
| `BROADCAST_LIVE_LIMIT` | 409 | 이미 진행 중인 방송이 있어요. | POST /live |
| `BROADCAST_READY_LIMIT` | 409 | 준비 중인 방송은 최대 5개까지 만들 수 있어요. | POST /broadcasts |
| `BROADCAST_FIELD_LOCKED` | 409 | 지금 방송 상태에서는 이 항목을 바꿀 수 없어요. | PATCH /broadcasts |
| `BROADCAST_NOT_LIVE` | 409 | 방송 중일 때만 할 수 있어요. | 채팅·후원·클러치·버스트·후원 주문·viewer-token·썸네일 |
| `BROADCAST_THUMBNAIL_INVALID` | 422 | 썸네일은 512KB 이하 JPEG만 올릴 수 있어요. | PUT thumbnail |
| `LIVE_PUBLISHER_ACTIVE` | 409 | 다른 기기에서 송출 중이에요. 이 기기로 전환할까요? | POST /live |
| `LIVE_WEBHOOK_INVALID_SIGNATURE` | 401 | (서버 간 호출) | LiveKit 웹훅 |
| `CHAT_BANNED_WORD` | 422 | FLUTCH 정책상 사용할 수 없는 표현이 포함되어 있어요. | 채팅 작성·수정 |
| `CHAT_REPEATED` | 429 | 같은 메시지를 반복할 수 없어요. | 채팅 작성 |
| `CHAT_NOT_AUTHOR` | 403 | 내가 쓴 채팅만 수정할 수 있어요. | PATCH /chats |
| `CHAT_EDIT_EXPIRED` | 409 | 작성 후 5분이 지나 수정할 수 없어요. | PATCH /chats |
| `CHAT_DELETED` | 409 | 삭제된 메시지예요. | PATCH /chats |
| `CHAT_DELETE_FORBIDDEN` | 403 | 작성자나 방송 소유자만 삭제할 수 있어요. | DELETE /chats |
| `MODERATION_NOT_OWNER` | 403 | 내 채널에서만 관리할 수 있어요. | bans, viewer chats |
| `MODERATION_SELF` | 422 | 자기 자신은 차단할 수 없어요. | PUT bans |
| `MODERATION_TIMED_OUT` | 403 | N분 후 채팅할 수 있어요. (+`retryAfterSec`) | 채팅 작성·수정 |
| `MODERATION_BANNED` | 403 | 이 채널에서 채팅이 제한되었어요. / 이 채널에서는 후원할 수 없어요. | 채팅, 유료 클러치, 버스트, 후원, 후원 주문 |
| `WALLET_INSUFFICIENT_BALANCE` | 409 | 비트가 부족해요. | 잔액 후원, 유료 클러치, 버스트 |
| `WALLET_BALANCE_LIMIT` | 409 | 보유 한도(3,000,000비트)를 넘을 수 없어요. | 주문, confirm |
| `DONATION_SELF` | 403 | 내 방송에는 후원할 수 없어요. | 후원, 후원 주문 |
| `DONATION_BANNED_WORD` | 422 | FLUTCH 정책상 사용할 수 없는 표현이 포함되어 있어요. | 후원·후원 주문 메시지 |
| `CLUTCH_SELF` | 403 | 내 방송에는 클러치할 수 없어요. | 클러치, 버스트 |
| `PAYMENT_NOT_NEEDED` | 409 | 잔액으로 바로 후원할 수 있어요. | 후원 주문 |
| `PAYMENT_ORDER_NOT_FOUND` | 404 | 주문을 찾을 수 없어요. | confirm |
| `PAYMENT_NOT_OWNER` | 403 | 내 주문만 확정할 수 있어요. | confirm |
| `PAYMENT_AMOUNT_MISMATCH` | 409 | 결제 금액이 주문과 달라요. | confirm |
| `PAYMENT_ORDER_EXPIRED` | 409 | 주문 시간이 지났어요. 다시 시도해 주세요. | confirm |
| `PAYMENT_ALREADY_CONFIRMED` | 409 | 이미 처리된 주문이에요. | confirm |
| `PAYMENT_APPROVAL_FAILED` | 409 | 결제 승인에 실패했어요. | confirm (토스 거절) |
| `PAYMENT_PROVIDER_UNAVAILABLE` | 502 | 결제 서비스에 연결할 수 없어요. 잠시 후 다시 시도해 주세요. | confirm (토스 통신 실패) |
| `RECORDING_EXPIRED` | 404 | 보관 기간이 지난 다시보기예요. | GET /recordings/{id} |
| `FOLLOW_SELF` | 422 | 자기 자신은 팔로우할 수 없어요. | PUT follow |
