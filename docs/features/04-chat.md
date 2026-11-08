# 04. 채팅 (스펙의 "댓글")

## 작성
- 로그인 + 차단되지 않음 + **방송이 LIVE일 때만**. 비로그인은 읽기만("로그인하고 채팅에 참여하세요").
- 최대 **200자**, **1초 1개**, 같은 메시지 **30초 안 3번** 거절("같은 메시지를 반복할 수 없어요").
- 욕설: **`***`로 가려서 표시**(원문은 DB 저장, 가린 결과만 응답). **FLUTCH 금지어: 전송 차단**("FLUTCH 정책상 사용할 수 없는 표현이 포함되어 있어요").
- 링크는 클릭되지 않는 일반 글자. 유니코드 이모지만.

## 수정·삭제
- 수정: **작성자만, 작성 후 5분 이내 + LIVE 중** → "(수정됨)".
- 삭제: **작성자 또는 방송 소유자, 언제든** → "삭제된 메시지입니다"로 대체.

## 표시
- 닉네임 + 메시지 + 배지: **스트리머 배지**, **후원자 배지**(이 방송에서 후원한 사람, 익명 후원 포함).
- 닉네임 색: 기본 프로필 배경색과 동일(사용자별 고정, DESIGN.md 색 역할 규칙 준수).
- 시간: 마우스를 올릴 때 표시.
- 후원 메시지는 강조 한 줄, 시스템 메시지(차단 알림 등)는 구분 스타일.

## 불러오기
- 입장 시 **최근 50개**, 위로 스크롤하면 이전 기록(**커서 페이지네이션**, OFFSET 미사용).
- ENDED·다시보기: 전체 목록(동기화 리플레이 없음).
- 인덱스: `(broadcast_id, id)`, `(streamer_id, user_id, id)` → 채팅에 `streamer_id` 함께 저장.

## 실시간
- WebSocket `chat.created / chat.updated / chat.deleted`. 서버 내 전달은 `tokio::broadcast`(Redis 교체 가능한 인터페이스).

## 시나리오 테스트
- `chat_live_only`: READY·ENDED 방송 작성 409
- `chat_guest_read_only`: 비로그인 조회 200, 작성 401
- `chat_edit_author_only_within_5min`
- `chat_delete_author_or_owner`: 다른 시청자 삭제 403
- `chat_rate_limit_and_repeat`
- `chat_profanity_masked_banned_word_blocked`
