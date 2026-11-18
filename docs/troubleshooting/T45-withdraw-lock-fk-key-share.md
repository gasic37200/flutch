# 탈퇴에 사용자 행 잠금을 FOR UPDATE로 넣으면 같은 사용자의 후원과 교착된다
- task: T45 · PR: (이 PR) · 날짜: 2026-11-18
- 영역: backend/auth, backend/wallet

## 증상
감사 지적: 결제 확정(`finalize`)이 `withdrawn_at`을 잠금 없이 읽어, 확인 직후 탈퇴가 끝나면 탈퇴 계정 지갑에 충전이 들어간다.
지시는 "탈퇴 시작에 `users FOR UPDATE`, 확정에 `FOR SHARE`(users → wallet 순서 통일)"였는데, 그대로 하면 교착 경로가 새로 생긴다.

## 원인
잔액 후원·클러치·버스트는 지갑 행을 먼저 잠그고(차감) 그 뒤 원장·후원 INSERT의 FK 확인으로 `users`에 **FOR KEY SHARE**를 잡는다
(지갑 → 사용자). 탈퇴가 사용자 행을 FOR UPDATE로 먼저 잡으면(사용자 → 지갑) FOR UPDATE는 KEY SHARE와 충돌해 순서가 엇갈린다.
"users → wallet 통일"은 FK 확인이 숨은 잠금이라 실제로는 성립하지 않는다.

## 해결
- 탈퇴: `SELECT 1 FROM users WHERE id = $1 FOR NO KEY UPDATE`를 맨 앞에. NO KEY UPDATE는 FOR SHARE(확정)와는 충돌하고 FK의 KEY SHARE와는 충돌하지 않는다.
- 확정: `user_withdrawn`에 `FOR SHARE`. 확정이 먼저면 탈퇴는 커밋까지 기다렸다가 새 잔액까지 소멸, 탈퇴가 먼저면 확정은 `withdrawn_at`을 보고 적립하지 않는다.
- 재현 시나리오 `payment_finalize_races_withdraw`: 지갑 행을 지운 사용자에 대해 별도 커넥션이 미커밋 `INSERT INTO wallets`를 들고 있으면, 확정은 탈퇴 확인 뒤 지갑 생성에서 멈추고 탈퇴의 `SELECT … FOR UPDATE`는 미커밋 행을 못 봐 멈추지 않는다(=고치기 전 경쟁 창). 고친 뒤엔 탈퇴가 사용자 잠금에서 기다린다.

## 재발 방지
- 잠금 순서를 정할 때 INSERT/UPDATE의 **FK 확인(KEY SHARE)** 도 잠금으로 센다. 부모 행을 "막기만" 하려면 FOR UPDATE 대신 FOR NO KEY UPDATE.
- 트랜잭션 중 `&state.db`로 두 번째 연결을 잡지 않는다(같은 T45: 풀 교착). 트랜잭션 안 조회는 `&mut *tx`.
