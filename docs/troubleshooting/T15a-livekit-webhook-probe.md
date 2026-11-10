# LiveKit 웹훅·참가자 형식 실측: 공용 LiveKit 웹훅이 안 닿아 별도 LiveKit 컨테이너로 확인
- task: T15a · PR: (이 PR) · 날짜: 2026-11-10
- 영역: backend/live, backend/livekit, deploy/compose.dev(확인 방법만)

## 증상
- 시청 토큰에 `hidden: true`를 넣기 전에 "hidden 참가자도 `participant_joined`/`participant_left` 웹훅이 오는가"를 확인해야 했다(안 오면 시청자 수가 깨진다).
- 웹훅만 찍는 작은 Node 서버를 WSL `8081`에 띄웠는데, 공용 LiveKit(`flutch-dev-livekit-1`) 로그에 `failed to send webhook ... dial tcp 192.168.65.254:8081: connect: connection refused`가 찍히고 웹훅이 오지 않았다. 컨테이너에서 `wget http://host.docker.internal:8081`도 거절. Windows에서 `localhost:8081`은 열렸다.

## 원인
- Docker Desktop의 `host.docker.internal`(192.168.65.254)은 Windows 호스트이고, WSL 포트는 Windows의 `127.0.0.1`에만 중계된다(`netstat`: `127.0.0.1:8081 LISTENING`). 처음 Node 서버는 `::`에만 열었고, `0.0.0.0`으로 다시 연 직후에도 몇 초간 거절됐다(중계가 새 포트를 늦게 잡는 것으로 보임). 나중에는 공용 LiveKit의 웹훅도 들어왔다 — 원인은 확정하지 못했다.
- 공용 LiveKit의 웹훅 설정(포트 하나)은 바꿀 수 없다(다른 task가 쓰는 중, 재시작 금지).

## 해결
- 확인용 LiveKit을 **별도 컨테이너**로 잠깐 띄웠다: 같은 이미지 `livekit/livekit-server:v1.13 --dev`, 포트 7890/7891/7892(udp), 웹훅 URL은 WSL `eth0` IP(`ip -4 addr show eth0`)의 `:8081`. 컨테이너에서 WSL IP로는 바로 닿는다. 방 이름도 `t15a-probe`로 달리 해 다른 worktree와 섞이지 않게 했다. 끝나고 컨테이너·서버를 지웠다.
- Browser 미리보기에서 livekit-client(UMD)로 송출 토큰·hidden 시청 토큰·같은 신원의 구독 전용 토큰으로 접속해 실측한 결과:
  - hidden 참가자도 `participant_joined`·`participant_left` 웹훅이 온다. `permission`은 `{"canSubscribe":true,"hidden":true}`(false인 `canPublish`는 생략). 송출자 클라이언트에는 hidden 참가자의 `participantConnected`가 오지 않았다.
  - 웹훅 `participant`에는 `sid`·`permission`이 joined·left 모두 들어 있다(camelCase).
  - `RoomService/ListParticipants`는 hidden 참가자도 포함하고, 필드는 **snake_case**(`permission.can_publish`)다. 웹훅(camelCase)과 이름이 달라 `livekit::Participant`는 `canPublish`에 `alias = "can_publish"`를 둔다.
  - 송출자가 있는 방에 같은 신원이 구독 전용으로 들어오면 송출자가 `DUPLICATE_IDENTITY`로 퇴장(left, `canPublish: true`)하고 구독 전용 참가자의 joined(`canPublish` 없음)가 온다.

## 재발 방지
- 웹훅 실측이 필요하면 공용 LiveKit 대신 위처럼 별도 포트의 LiveKit 컨테이너 + WSL IP 웹훅으로 확인한다(다른 worktree와 방·웹훅이 섞이지 않는다, T08b 문제도 피함).
- `livekit.rs` 단위 테스트 `participant_parses_webhook_and_server_api`가 두 형식을 고정한다.
