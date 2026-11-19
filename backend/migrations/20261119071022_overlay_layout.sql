-- 스트리머가 스튜디오 "배치 편집"으로 정한 오버레이 위치(영상 기준 %). NULL이면 프론트 기본값.
-- 모양 {alert:{x,y}, cam:{x,y,w}, chat:{x,y,w}} — 검증은 서버 코드(api.md §3)가 한다.
-- shortcut: pip_corner는 overlay_layout 이전 값이라 남겨 둔다(파괴적 변경 금지), 제출 후 정리.
ALTER TABLE broadcasts ADD COLUMN overlay_layout JSONB;
