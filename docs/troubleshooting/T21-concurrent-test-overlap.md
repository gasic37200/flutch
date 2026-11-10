# 동시 confirm 시나리오가 단독 실행하면 겹치지 않아 실패
- task: T21 · PR: (이 PR) · 날짜: 2026-11-10
- 영역: backend/tests (payment)

## 증상
`payment_concurrent_confirm_credits_once`가 `cargo test --test scenarios payment` 전체로는 통과하는데, 단독 실행(`payment_concurrent`)하면 매번 `confirm_calls.len()` 1 ≠ 2로 실패.

## 원인
`#[sqlx::test]` 런타임은 단일 스레드라 `tokio::join!`의 두 요청은 await 지점마다 번갈아 돈다. 앞 요청이 승인·확정까지 빨리 끝내면 뒤 요청은 사전 확인에서 이미 `done`인 주문을 보고(멱등 응답) 토스까지 가지 않는다. 다른 테스트와 함께 돌 때는 DB가 느려 우연히 겹쳤을 뿐이다. 적립 1회 검증은 맞았지만 "실제로 겹쳤다"는 보장이 없었다.

## 해결
`FakeToss.confirm_delay_ms`(승인 응답 지연)를 추가하고 이 시나리오만 200ms로 둔다. 두 요청이 모두 사전 확인을 지나 토스 승인까지 간 뒤 확정 트랜잭션(`FOR UPDATE`)에서 만나는 것을 `confirm_calls == 2`로 확인한다.

## 재발 방지
동시성 시나리오는 "겹쳤다"를 단언(호출 수 등)하고, 겹침은 fake의 지연이나 트랜잭션 안 sleep(T20 `wallet_concurrent_debit_only_one_succeeds`)으로 만든다. 단독 실행으로도 여러 번 돌려 본다.
