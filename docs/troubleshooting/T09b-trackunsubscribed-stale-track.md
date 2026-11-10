# 웹캠(트랙)을 내려도 시청 화면에 남음: TrackUnsubscribed 시점에 pub.track이 아직 있음
- task: T09b · PR: (이 PR) · 날짜: 2026-11-10
- 영역: frontend/live (`useViewer`)

## 증상
스튜디오에서 웹캠 PIP를 끄면 서버·스튜디오는 카메라 트랙을 내렸는데, 시청 화면에는 PIP 버튼·카메라 아이콘이 남고 영상은 비어 있었다(`tracks.camera`가 살아 있음).

## 원인
livekit-client 2.23은 `RemoteTrackPublication.setTrack(undefined)`에서 `Unsubscribed`를 **먼저 발행하고 그 뒤에 `track`을 비운다**. `useViewer`의 `sync()`가 `TrackUnsubscribed` 핸들러 안에서 `pub.track`을 읽으면 떠나는 트랙이 그대로 남는다. 마지막 트랙이 빠지는 경우 뒤따르는 이벤트도 없어 상태가 고쳐지지 않는다.

## 해결
`TrackUnsubscribed`의 인자(떠나는 트랙)를 `sync(gone)`에 넘겨 결과에서 뺀다.

## 재발 방지
LiveKit 이벤트 핸들러 안에서 "현재 상태"(publication 목록·`pub.track`)를 다시 읽을 때는 이벤트가 상태 변경 전에 발행될 수 있음을 가정하고, 이벤트 인자로 받은 대상을 직접 제외한다.
