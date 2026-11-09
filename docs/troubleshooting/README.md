# 트러블슈팅 기록

> 서브에이전트가 task를 마칠 때, 작업 중 막혔던 것·예상과 달랐던 것·시행착오가 있었으면 기록한다. 같은 영역을 작업하기 전에 이 목록을 먼저 본다.

## 규칙
- 파일 하나 = 문제 하나. 이름: `T<번호>-<짧은-영문-설명>.md` (예: `T08-livekit-webhook-signature.md`)
- 아래 목록 표에 한 줄 추가(같은 PR에서).
- 해결 과정에서 규칙이 생기면 "재발 방지"에 적고, 문서(AGENTS·features·architecture) 반영이 필요하면 같은 PR에 포함.
- 비밀값·토큰·개인정보는 적지 않는다.

## 양식
```markdown
# <문제 한 줄 요약>
- task: Txx · PR: #nn · 날짜: YYYY-MM-DD
- 영역: <backend/live, frontend/studio, deploy ...>

## 증상
<무엇이 어떻게 안 됐나. 에러 메시지 핵심 줄>

## 원인
<왜 그랬나>

## 해결
<무엇을 바꿨나. 핵심 코드·설정 한두 줄>

## 재발 방지
<규칙·체크 항목, 반영한 문서>
```

## 목록
| 파일 | task | 요약 |
|---|---|---|
| [T01-livekit-webhook-wsl.md](T01-livekit-webhook-wsl.md) | T01 | 로컬 LiveKit 웹훅은 WSL 백엔드에 닿음(확인), compose는 루트 .env를 안 읽어 `--env-file .env` 필요 |
| [T01-reqwest-ring-backend.md](T01-reqwest-ring-backend.md) | T01 | reqwest 0.13은 rustls 기본이 aws-lc → ring 쓰려고 0.12 고정 |
