# LiveKit 토큰의 `canPublish`·`canPublishData`는 빠지면 true — 시청 토큰에 false를 명시
- task: T08a · PR: #38 · 날짜: 2026-11-09
- 영역: backend/livekit

## 증상
실제로 겪기 전에 확인한 위험. 시청 토큰을 "`roomJoin, canSubscribe`만"으로 만들면(다른 권한 키를 생략) 시청자도 송출·데이터 메시지를 보낼 수 있게 된다.

## 원인
LiveKit 서버의 `VideoGrant`는 `canPublish`·`canPublishData`·`canSubscribe`가 포인터(`*bool`)이고 **없으면 true**로 해석한다. `canPublishSources`가 비어 있으면 모든 소스 허용.

## 해결
`livekit::access_token`이 항상 `canPublish`(시청 false)·`canPublishData: false`·`canSubscribe: true`를 명시하고, 송출 토큰에만 `canPublishSources: [camera, microphone, screen_share, screen_share_audio]`를 넣는다.
로컬 LiveKit(`--dev`)에서 발급 토큰으로 `/rtc` 접속 후 `RoomService/ListParticipants`로 확인: 송출자 `can_publish=true`·소스 4개, 시청자 `can_publish=false`, 둘 다 `can_publish_data=false`.

## 재발 방지
- 시나리오 `live_start_owner_only`가 시청 토큰의 `canPublish == false`·`canPublishData == false`를 확인한다.
- api.md §3-1 권한 줄에 "`canPublish=false` 명시" 반영.
