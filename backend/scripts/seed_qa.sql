-- 로컬 QA 더미 데이터. **로컬 QA DB 전용 — 운영 DB에서 실행 금지**(어떤 자동 경로도 이 파일을 실행하지 않는다).
-- 넣는 것: 시드 3계정에 테스트 지급(grant) 100,000비트씩, viewer → streamer1·streamer2 팔로우,
--          스트리머별 READY 방송, 후원을 받은 ENDED 방송 1개씩(녹화 없음), viewer의 후원·유료 클러치 기록.
-- 전제: 백엔드를 한 번 실행해 시드 계정(streamer1·streamer2·viewer)이 만들어져 있어야 한다. 없으면 에러로 멈춘다.
-- 한 번만 들어간다: viewer에게 grant 원장이 이미 있으면 아무 것도 하지 않는다(NOTICE만). DO 블록 하나라 한 트랜잭션.
-- 실행 예(저장소 루트, DB 이름은 .env의 DATABASE_URL):
--   psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f backend/scripts/seed_qa.sql
--   docker compose -f deploy/compose.dev.yml exec -T postgres psql -U flutch -d flutch_dev -v ON_ERROR_STOP=1 < backend/scripts/seed_qa.sql
DO $seed$
DECLARE
    v_s1     BIGINT;
    v_s2     BIGINT;
    v_viewer BIGINT;
    v_e1     BIGINT;
    v_e2     BIGINT;
    v_uid    BIGINT;
    v_lid    BIGINT;
    v_at     TIMESTAMPTZ;
    r        RECORD;
BEGIN
    SELECT id INTO v_s1 FROM users WHERE login_id = 'streamer1';
    SELECT id INTO v_s2 FROM users WHERE login_id = 'streamer2';
    SELECT id INTO v_viewer FROM users WHERE login_id = 'viewer';
    IF v_s1 IS NULL OR v_s2 IS NULL OR v_viewer IS NULL THEN
        RAISE EXCEPTION '시드 계정(streamer1·streamer2·viewer)이 없습니다. 백엔드를 한 번 실행한 뒤 다시 실행하세요.';
    END IF;
    IF EXISTS (SELECT 1 FROM ledger_entries WHERE user_id = v_viewer AND type = 'grant') THEN
        RAISE NOTICE 'QA 시드가 이미 들어 있어 건너뜁니다.';
        RETURN;
    END IF;

    -- 테스트 지급: 지갑 갱신과 원장 추가를 한 문장으로(wallet::credit과 같은 패턴). 상한 초과면 wallets CHECK가 막는다.
    FOREACH v_uid IN ARRAY ARRAY[v_s1, v_s2, v_viewer] LOOP
        WITH w AS (
            INSERT INTO wallets (user_id, balance) VALUES (v_uid, 100000)
            ON CONFLICT (user_id) DO UPDATE SET balance = wallets.balance + EXCLUDED.balance
            RETURNING balance
        )
        INSERT INTO ledger_entries (user_id, type, amount, balance_after, created_at)
        SELECT v_uid, 'grant', 100000, w.balance, now() - interval '3 days' FROM w;
    END LOOP;

    INSERT INTO follows (follower_id, streamer_id) VALUES (v_viewer, v_s1), (v_viewer, v_s2)
    ON CONFLICT DO NOTHING;

    -- shortcut: READY 5개 상한은 앱 규칙이라 기존 READY가 많은 DB에선 넘을 수 있다, 문제가 되면 새 DB에서 실행한다.
    INSERT INTO broadcasts (owner_id, title, category, tags) VALUES
        (v_s1, '주말 랭크 같이 올려요', 'fps', ARRAY['발로란트', '랭크']),
        (v_s1, '공포게임 정주행 1일차', 'horror', ARRAY['공포', '정주행']),
        (v_s1, '새벽 수다 방송', 'just_chatting', '{}'),
        (v_s2, '버추얼 노래 방송', 'virtual', ARRAY['노래', '버튜버']),
        (v_s2, '보스 러시 도전기', 'rpg', ARRAY['보스러시', '소울라이크', '도전']);

    INSERT INTO broadcasts (owner_id, status, title, category, tags, record, started_at, ended_at, peak_viewers)
    VALUES (v_s1, 'ENDED', '랭크 하이라이트 방송', 'fps', ARRAY['랭크'], false,
            now() - interval '2 days', now() - interval '2 days' + interval '2 hours', 42)
    RETURNING id INTO v_e1;
    INSERT INTO broadcasts (owner_id, status, title, category, tags, record, started_at, ended_at, peak_viewers)
    VALUES (v_s2, 'ENDED', '주말 축구 같이 보기', 'sports', ARRAY['축구'], false,
            now() - interval '1 day', now() - interval '1 day' + interval '90 minutes', 17)
    RETURNING id INTO v_e2;

    -- 후원: 조건부 차감 + 원장(wallet::debit과 같은 패턴) → donations가 그 원장을 가리킨다. 유료 클러치는 1회 1비트.
    FOR r IN
        SELECT * FROM (VALUES
            (v_e1, 'donation', 5000::bigint, false, '오늘 에임 미쳤어요!'::text, 10),
            (v_e1, 'donation', 1000::bigint, true, '익명으로 응원합니다', 25),
            (v_e1, 'clutch', 1::bigint, false, NULL, 30),
            (v_e1, 'clutch', 1::bigint, false, NULL, 31),
            (v_e1, 'donation', 30000::bigint, false, '하이라이트 클립 기대할게요', 100),
            (v_e2, 'donation', 10000::bigint, false, '다음 경기도 같이 봐요', 15),
            (v_e2, 'clutch', 1::bigint, false, NULL, 40)
        ) AS t (broadcast_id, kind, amount, anonymous, message, minutes)
    LOOP
        WITH w AS (
            UPDATE wallets SET balance = balance - r.amount
            WHERE user_id = v_viewer AND balance >= r.amount
            RETURNING balance
        )
        INSERT INTO ledger_entries (user_id, type, amount, balance_after, broadcast_id, created_at)
        SELECT v_viewer, r.kind, -r.amount, w.balance, r.broadcast_id,
               b.started_at + make_interval(mins => r.minutes)
        FROM w, broadcasts b WHERE b.id = r.broadcast_id
        RETURNING id, created_at INTO v_lid, v_at;
        IF v_lid IS NULL THEN
            RAISE EXCEPTION 'viewer 잔액이 부족해 후원 시드를 넣을 수 없습니다.';
        END IF;
        INSERT INTO donations (broadcast_id, donor_id, kind, amount, anonymous, message, ledger_entry_id, created_at)
        VALUES (r.broadcast_id, v_viewer, r.kind, r.amount, r.anonymous, r.message, v_lid, v_at);
    END LOOP;
END
$seed$;
