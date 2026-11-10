-- SUPER CLUTCH 회전(T29): 마지막으로 히어로에 노출된 슬롯 시작 시각. NULL = 아직 노출 전(다음 순서로 먼저 나간다).
-- 현재 슬롯 = 진행 중(LIVE·미만료) 중 shown_at이 가장 늦은 것. 1초 작업과 홈 히어로 조회가 이 값으로 진행한다(재시작 안전).
ALTER TABLE super_clutches ADD COLUMN shown_at TIMESTAMPTZ;
-- 진행 중 목록(expires_at > now()) 조회
CREATE INDEX super_clutches_expires_idx ON super_clutches (expires_at);
