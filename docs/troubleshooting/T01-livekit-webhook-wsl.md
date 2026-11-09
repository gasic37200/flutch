# 로컬 LiveKit 웹훅이 WSL 백엔드에 닿는지 · compose가 루트 .env를 읽지 않음
- task: T01 · PR: (생성 후 기입) · 날짜: 2026-11-09
- 영역: deploy/compose.dev, backend

## 증상
1. 확인 전 우려: Docker Desktop의 `host.docker.internal`은 Windows 호스트를 가리켜, 컨테이너(LiveKit)에서 WSL Ubuntu의 백엔드로 웹훅이 안 닿을 수 있음.
2. 실제로 겪은 것: 저장소 루트에서 `docker compose -f deploy/compose.dev.yml up -d`를 하면 루트 `.env`의 `POSTGRES_PORT`·`BACKEND_PORT`가 무시되고 기본값(5432, 8080)으로 뜸. 이 PC는 5432를 다른 프로젝트 Postgres가 쓰고 있어 충돌.

## 원인
1. Docker Desktop(WSL 연동)에서는 `host.docker.internal`(192.168.65.254)로 들어온 연결이 WSL에서 `0.0.0.0`에 열린 포트까지 전달된다. 따라서 Desktop에서는 문제없음.
2. compose는 변수 치환용 `.env`를 **compose 파일이 있는 폴더(`deploy/`)**에서 찾는다. 루트 `.env`는 읽지 않는다.

## 해결
- 확인 방법: 백엔드(`BACKEND_PORT=8081`)를 WSL에서 띄우고
  - `docker exec flutch-dev-livekit-1 wget -qO- http://host.docker.internal:8081/api/v1/health` → `{"status":"ok"}`
  - LiveKit API로 방을 만들고 지우자 LiveKit 로그에 `sent webhook ... room_started / room_finished ... url: http://host.docker.internal:8081/api/v1/livekit/webhook`, 백엔드 접근 로그에 POST 2건(웹훅 라우트가 아직 없어 404, Origin 검사 예외는 정상 통과).
  - `extra_hosts` 없는 컨테이너에서도 같은 결과 → Docker Desktop에서는 필수가 아님.
- 그래도 compose.dev의 livekit에 `extra_hosts: ["host.docker.internal:host-gateway"]`를 둔다. Docker Desktop에서는 같은 주소로 풀려 무해하고, WSL에 Docker Engine을 직접 설치한 환경(또는 리눅스 PC)에서는 이것이 없으면 이름이 풀리지 않는다.
- compose 실행은 항상 `--env-file .env`를 붙인다: `docker compose --env-file .env -f deploy/compose.dev.yml up -d`. 포트는 `${POSTGRES_PORT:-5432}`·`${BACKEND_PORT:-8080}`로 `.env`에서 조정.

## 재발 방지
- README "실행 방법(개발)"의 compose 명령에 `--env-file .env` 반영.
- AGENTS.md §3의 compose 명령도 같은 형태로 바꿔야 함(메인 세션에 요청).
