# kro.kr 인증서 발급이 Let's Encrypt 주간 한도로 거절됨
- task: T07 · PR: (T07 PR) · 날짜: 2026-11-09
- 영역: deploy (init-cert.sh, certbot)

## 증상
`certbot certonly` → `too many certificates (50) already issued for "kro.kr" in the last 168h0m0s, retry after ...`

## 원인
Let's Encrypt의 "등록 도메인당 주간 50개" 한도가 `kro.kr` 전체(내도메인을 쓰는 모든 사용자)에 걸린다. 우리 쪽 시도 횟수와 무관하며, 한도는 에러 메시지의 `retry after` 시각에 풀린다. 챌린지 도달 여부와는 별개로 발급 요청 단계에서 거절된다.

## 해결
메시지의 시각 이후 `./init-cert.sh`를 다시 실행(11/09 18:30·18:46 KST 두 번 거절, 한도가 다른 사용자 발급으로 다시 차서 해제 시각이 22:11 KST로 밀림). 거절돼도 서버 상태는 남아 그대로 재시도할 수 있다. 챌린지 경로(임시 80번 nginx + webroot)는 `--staging` 발급으로 확인함(staging은 별도 한도).

## 재발 방지
- 제출 직전(11/18)에 처음 발급하지 말고 일찍 발급한다. 갱신은 만료 30일 전부터 시도하므로 한도에 걸려도 여유가 있다.
- 갱신이 한도에 걸려 실패해도 nginx는 기존 인증서로 계속 동작한다(만료 전까지 매 12시간 재시도).
