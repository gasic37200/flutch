-- 원장 유형 grant(테스트 지급, T34): 로컬 QA 시드(backend/scripts/seed_qa.sql) 전용 적립. 주문·방송을 참조하지 않는다.
-- 추가적 변경만: 허용 유형과 "+는 charge만" 규칙을 grant까지 넓힌다(기존 행은 그대로 통과).
-- payment_order_id 규칙(check1)은 그대로 — grant는 charge가 아니므로 주문 없이(NULL) 통과한다.
ALTER TABLE ledger_entries
    DROP CONSTRAINT ledger_entries_type_check,
    ADD CONSTRAINT ledger_entries_type_check
        CHECK (type IN ('charge', 'grant', 'donation', 'clutch', 'burst', 'withdraw_forfeit')),
    DROP CONSTRAINT ledger_entries_check,
    ADD CONSTRAINT ledger_entries_sign_check CHECK ((type IN ('charge', 'grant')) = (amount > 0));
