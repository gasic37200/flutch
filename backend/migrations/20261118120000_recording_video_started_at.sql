-- 영상 0초의 실제 시각(Egress 파일 시작). 송출 시작(broadcasts.started_at)보다 몇 초 늦다 — 다시보기 동기화·클립 오프셋 기준.
ALTER TABLE recordings ADD COLUMN video_started_at TIMESTAMPTZ;
