# 공용 로컬 LiveKit에서 worktree끼리 방 이름(`broadcast-<id>`)과 웹훅이 섞인다
- task: T08b · PR: (이 PR) · 날짜: 2026-11-09
- 영역: deploy/compose.dev, backend/live

## 증상
실제 웹훅 확인을 위해 웹훅 대상 포트(8081)에 이 worktree 백엔드를 띄웠더니, 동시에 작업 중인 다른 worktree(프론트 task)의 LiveKit 방 `broadcast-1` 이벤트까지 들어왔다. 각 worktree DB의 방송 id가 1부터 시작하므로 같은 방 이름을 서로 다른 방송이 쓴다. 웹훅을 받는 백엔드가 자기 DB 기준으로 유예 종료 → `DeleteRoom`을 부르면 다른 worktree의 같은 이름 방을 닫을 수 있다.

## 원인
compose.dev의 LiveKit 하나를 모든 worktree가 공유하고, 웹훅 URL은 compose를 마지막으로 띄운 `.env`의 `BACKEND_PORT` 하나뿐이다. 방 이름에 worktree 구분이 없다.

## 해결
- 확인은 짧게 끝내고 서버를 바로 내렸다(이 PR의 백엔드가 지운 방은 `broadcast-2`·`broadcast-3`, 다른 task는 `broadcast-1`을 쓰고 있어 겹치지 않았다).
- 웹훅·자동 종료를 실제로 확인하려면: 다른 task가 LiveKit을 쓰지 않을 때, 웹훅 포트의 백엔드 하나만 띄우고 확인한다. 수동 확인용으로 `LIVE_GRACE_SECS=15`처럼 유예를 줄일 수 있다.

## 재발 방지
- 공용 LiveKit으로 웹훅을 확인하는 task는 다른 LiveKit 사용 task와 시간을 나눈다(메인 세션 배정 시 고려).
- 운영은 LiveKit Cloud 프로젝트 하나 = 서버 하나라 해당 없음.
