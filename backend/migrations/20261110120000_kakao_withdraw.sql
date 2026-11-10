-- 탈퇴한 카카오 계정은 kakao_id를 비워 같은 카카오 계정이 새 계정으로 다시 가입할 수 있게 한다(features/01).
-- 기존 CHECK는 kakao_id NULL을 막으므로 탈퇴 행은 예외로 둔다.
ALTER TABLE users DROP CONSTRAINT users_check;
ALTER TABLE users ADD CONSTRAINT users_identity_check
    CHECK ((login_id IS NOT NULL AND password_hash IS NOT NULL) OR kakao_id IS NOT NULL OR withdrawn_at IS NOT NULL);
