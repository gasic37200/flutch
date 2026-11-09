# API 명세

> 상태: **초안**. roadmap **T02b(API 계약)**에서 엔드포인트별 요청·응답·권한·에러 코드를 채우고 머지한 뒤에 프론트·백엔드 병렬 task를 시작한다. 엔드포인트를 바꾸는 PR은 이 문서를 같은 PR에서 갱신한다.

## 규칙
- 접두사 `/api/v1`. 리소스는 경로, 필터·검색·페이지는 쿼리, 데이터는 JSON 본문.
- 인증: httpOnly 세션 쿠키. 변경 요청(POST/PATCH/DELETE)은 `Origin` 검사.
- 에러 형식: `{ "error": { "code": "BROADCAST_NOT_OWNER", "message": "사용자에게 보여줄 한국어 문장", "request_id": "…" } }`. 응답 헤더 `x-request-id`도 같은 값.
- 에러 코드: `SCREAMING_SNAKE_CASE`, `<도메인>_<이유>`. 공통: `UNAUTHENTICATED`, `FORBIDDEN`, `NOT_FOUND`, `VALIDATION_FAILED`(+`fields: {name: 이유}`), `RATE_LIMITED`(+`retry_after_sec`), `CONFLICT`, `INTERNAL`. 도메인 코드 목록은 T02b에서 이 문서 아래 표에 채운다.
- 페이지네이션: 커서 방식. 요청 `?cursor=<불투명 문자열>&limit=<기본 20, 최대 50>`, 응답 `{ "items": [...], "next_cursor": "…" | null }`. 채팅은 `?before=<id>`로 이전 기록.
- 숫자 id·금액은 JSON number(i64). 시간은 RFC 3339 UTC 문자열.
- 상태 코드: 401 미인증, 403 권한 없음, 404 없음, 409 상태 충돌(잘못된 방송 상태 전이 등), 422 검증 실패, 429 제한 초과.

## 엔드포인트 (예정)
| 메서드 | 경로 | 권한 | 설명 |
|---|---|---|---|
| POST | `/auth/signup` | 비로그인 | 회원가입 |
| POST | `/auth/login` | 비로그인 | 로그인 |
| POST | `/auth/test-login` | 비로그인 | 테스트 계정 바로 로그인 |
| POST | `/auth/logout` | 로그인 | 로그아웃 |
| GET | `/me` | 로그인 | 내 정보 |
| PATCH | `/me` | 로그인 | 닉네임(30일 제한)·비밀번호 변경 |
| GET | `/broadcasts` | 누구나 | 목록 (`?status=&category=&q=`) |
| POST | `/broadcasts` | 로그인 | 방송 등록 |
| GET | `/broadcasts/{id}` | 누구나 | 상세 |
| PATCH | `/broadcasts/{id}` | 소유자 | 제목·카테고리 등 수정 (LIVE 중 가능) |
| DELETE | `/broadcasts/{id}` | 소유자 | 삭제 (LIVE면 강제 종료 후) |
| POST | `/broadcasts/{id}/live` | 소유자 | 송출 시작 → publish 토큰 |
| DELETE | `/broadcasts/{id}/live` | 소유자 | 송출 종료 |
| POST | `/broadcasts/{id}/viewer-token` | 누구나 | 시청용 subscribe 토큰 |
| PUT | `/broadcasts/{id}/thumbnail` | 소유자 | 썸네일 업로드 |
| GET | `/broadcasts/{id}/chats` | 누구나 | 채팅 목록 |
| POST | `/broadcasts/{id}/chats` | 로그인·미차단·LIVE | 채팅 작성 |
| PATCH | `/chats/{id}` | 작성자 | 채팅 수정 |
| DELETE | `/chats/{id}` | 작성자·방송 소유자 | 채팅 삭제 |
| GET/POST/DELETE | `/users/{id}/bans` | 해당 스트리머 | 차단 목록·차단·해제 |
| POST | `/broadcasts/{id}/clutches` | 로그인 | 클러치 버튼 (무료/유료) |
| POST | `/broadcasts/{id}/bursts` | 로그인 | SUPER BURST |
| POST | `/payments/orders` | 로그인 | 결제 대기 주문 (충전 또는 바로 후원) |
| POST | `/payments/confirm` | 로그인 | 토스 승인 확정 |
| POST | `/payments/webhook` | 토스 | 웹훅 (멱등) |
| POST | `/broadcasts/{id}/donations` | 로그인·LIVE | 잔액 후원 |
| GET | `/broadcasts/{id}/donations` | 누구나 (소유자는 익명 실명) | 후원 내역·합계 |
| GET | `/me/wallet` | 로그인 | 잔액·원장 내역 |
| GET | `/recordings` · `/recordings/{id}` | 누구나 | 다시보기 |
| PUT/DELETE | `/users/{id}/follow` | 로그인 | 팔로우·취소 |
| GET | `/users/{id}` | 누구나 | 스트리머 페이지 |
| GET | `/home/hero` | 누구나 | 히어로 상태: 현재 방송, 진행 중 SUPER CLUTCH 목록(최대 5), `next_switch_at`, 없으면 게이지 1위 (홈 10초 폴링) |
| POST | `/livekit/webhook` | LiveKit | 참가·퇴장·방 종료·Egress 완료 (서명 검증) |
| DELETE | `/me` | 로그인 | 회원 탈퇴 (비밀번호·잔액 소멸 확인) |
| GET | `/users/{id}/chats` | 해당 스트리머 | 이 시청자가 내 방송들에서 쓴 채팅 (커서) |
| GET | `/pulse` | 로그인 | PULSE: 팔로우 채널 라이브·오프라인 |
| GET | `/health` | 누구나 | 헬스 체크 |

## WebSocket `/api/v1/broadcasts/{id}/events`
- 연결 시 `Origin` 검사, 세션 쿠키가 있으면 사용자 식별(없으면 익명). 이벤트는 연결별로 걸러 보낸다(익명 후원자 실명은 소유자 연결에만).
- 서버 → 클라이언트 이벤트: `chat.created` `chat.updated` `chat.deleted` `chat.system`(차단 알림) `broadcast.updated`(제목·카테고리·태그·PIP 위치) `broadcast.ended` `broadcast.reconnecting` `broadcast.resumed` `broadcast.duration_warning`(종료 10분 전, 소유자만) `viewer.count` `donation.created` `donation.skipped` `clutch.gauge` `clutch.super`
- 클라이언트 → 서버는 보내지 않는다(쓰기는 모두 REST). 화면 공유 중단·웹캠 켜짐 같은 트랙 변화는 시청자가 LiveKit 이벤트로 직접 감지한다. 
