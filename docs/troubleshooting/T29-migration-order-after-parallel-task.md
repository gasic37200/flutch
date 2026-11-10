# 병렬 task의 마이그레이션보다 버전이 앞서는 마이그레이션 (rebase 후)
- task: T29 · PR: (이 PR) · 날짜: 2026-11-10
- 영역: backend/migrations

## 증상
`sqlx migrate add` 대신 현재 UTC 시각으로 `20261110074430_super_clutch_rotation.sql`을 만들었는데, PR 전 `main`으로 rebase하니 먼저 머지된 T28의 `20261110120000_kakao_withdraw.sql`보다 버전이 앞섰다. 운영 DB에는 T28 것이 이미 적용돼 있어 "적용된 최신 버전보다 오래된 미적용 마이그레이션"이 된다.

## 원인
마이그레이션 버전은 만든 시각(UTC)이다. 병렬 task가 같은 날 더 큰 버전(여기서는 KST 기준으로 잡은 것)을 먼저 머지하면 나중에 머지되는 쪽이 순서상 끼어든다. 실행 순서가 머지 순서와 달라지고, 도구에 따라 순서 밖 적용을 거부할 수 있다.

## 해결
rebase 직후 `ls backend/migrations`로 확인하고 내 파일을 main의 마지막 버전보다 큰 `20261110130000_super_clutch_rotation.sql`로 바꿨다(아직 머지 전이라 이름 변경 가능). 로컬 task DB는 지우고 다시 만들어(`sqlx database drop -y && sqlx database create && sqlx migrate run`) 새 버전으로 적용했다.

## 재발 방지
PR 전 rebase 뒤 마이그레이션 파일 버전이 main의 마지막 것보다 큰지 확인한다. 작으면 머지 전에 이름을 바꾼다(머지된 마이그레이션은 고치지 않는다).
