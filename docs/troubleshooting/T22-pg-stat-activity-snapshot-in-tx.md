# 동시 후원 시나리오가 "겹침 대기"에서 멈춤 (pg_stat_activity 스냅숏)
- task: T22 · PR: (이 PR) · 날짜: 2026-11-10
- 영역: backend/tests (donation)

## 증상
`donation_balance_conditional_debit`(HTTP 잔액 후원 2건 동시)이 단독 실행에서도 5번 중 2번꼴로 "두 요청이 잠금에서 겹치지 않음"으로 60초 뒤 실패. 처음엔 테스트 풀 고갈(`PoolTimedOut`)도 섞여 보였다.

## 원인
두 가지가 겹쳤다.
1. 겹침 확인을 잠금을 쥔 **트랜잭션 안에서** `pg_stat_activity`로 반복 조회했다. 통계 뷰는 트랜잭션의 첫 조회 때 스냅숏을 떠서 같은 트랜잭션에서는 값이 바뀌지 않는다(`stats_fetch_consistency = cache`). 첫 조회 시점에 대기 중인 요청이 0~1개면 영원히 2가 안 된다.
2. 처음 버전은 잠금 트랜잭션과 확인 쿼리를 `#[sqlx::test]` 풀에서 꺼냈다. 테스트 풀은 테스트당 5개, 병렬 테스트 전체가 부모 풀 20개를 나눠 쓰므로, 잠금을 쥔 채 커넥션을 더 기다리면 부하가 클 때 막힌다.

## 해결
- 잠금·확인은 풀 밖의 별도 커넥션(`PgConnection::connect_with(&db.connect_options())`)으로 하고, 매 조회 전에 `SELECT pg_stat_clear_snapshot()`.
- 지갑 행을 `FOR UPDATE`로 잡아 두고 `wait_event_type = 'Lock'`인 세션이 2개가 될 때(두 요청 모두 차감 UPDATE에서 대기 = 실제 겹침) 커밋해 풀어 준다.

## 재발 방지
트랜잭션 안에서 `pg_stat_activity`·`pg_locks` 같은 통계로 상태를 폴링하면 매번 `pg_stat_clear_snapshot()`. 동시성 테스트에서 잠금을 쥐는 쪽은 테스트 풀을 쓰지 않는다.
