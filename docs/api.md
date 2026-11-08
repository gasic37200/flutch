# API 명세

> 상태: **초안**. 기반 단계에서 엔드포인트별 요청·응답·권한을 채운다. 엔드포인트를 바꾸는 PR은 이 문서를 같은 PR에서 갱신한다.

## 규칙
- 접두사 `/api/v1`. 리소스는 경로, 필터·검색·페이지는 쿼리, 데이터는 JSON 본문.
- 인증: httpOnly 세션 쿠키. 변경 요청(POST/PATCH/DELETE)은 `Origin` 검사.
- 에러 형식: `{ "error": { "code": "BROADCAST_NOT_OWNER", "message": "..." } }`
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
| GET | `/clutch/queue` | 누구나 | 히어로·대기열 |
| POST | `/livekit/webhook` | LiveKit | 송출자 퇴장·방 종료 |
| GET | `/health` | 누구나 | 헬스 체크 |

## WebSocket `/api/v1/broadcasts/{id}/events`
서버 → 클라이언트 이벤트: `chat.created` `chat.updated` `chat.deleted` `broadcast.updated` `broadcast.ended` `broadcast.reconnecting` `viewer.count` `donation.created` `clutch.gauge` `clutch.super` 
