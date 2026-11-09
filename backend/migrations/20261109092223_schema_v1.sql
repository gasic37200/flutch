-- 스키마 v1: 모든 도메인 테이블. 근거는 docs/architecture.md §7, docs/features/01~10.
-- 규칙: ID는 BIGINT identity, 비트는 BIGINT, 시간은 timestamptz(UTC).

-- 추가만 하는 테이블(원장·차단 이력)의 수정·삭제를 DB에서 막는다.
CREATE FUNCTION reject_modification() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION '% is append-only', TG_TABLE_NAME;
END $$;

-- ===== 계정 =====
CREATE TABLE users (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- 영구 유니크(탈퇴해도 행을 지우지 않으므로 재사용 불가). 소문자만 저장해 대소문자 무시 중복을 보장.
    login_id            TEXT UNIQUE CHECK (login_id ~ '^[a-z0-9]{4,20}$'),
    password_hash       TEXT,
    kakao_id            BIGINT UNIQUE,
    nickname            TEXT NOT NULL,
    nickname_changed_at TIMESTAMPTZ,
    failed_login_count  INTEGER NOT NULL DEFAULT 0,
    locked_until        TIMESTAMPTZ,
    withdrawn_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((login_id IS NOT NULL AND password_hash IS NOT NULL) OR kakao_id IS NOT NULL)
);
-- 활성 사용자끼리만 유니크. 탈퇴 후 30일 보류는 시간 조건이라 인덱스로 못 걸어 가입·변경 시 앱이 확인한다.
CREATE UNIQUE INDEX users_nickname_active_key ON users (lower(nickname)) WHERE withdrawn_at IS NULL;

CREATE TABLE sessions (
    token_hash   BYTEA PRIMARY KEY CHECK (octet_length(token_hash) = 32),
    user_id      BIGINT NOT NULL REFERENCES users (id),
    remember     BOOLEAN NOT NULL,
    expires_at   TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);

-- ===== 방송 =====
CREATE TABLE broadcasts (
    id                 BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    owner_id           BIGINT NOT NULL REFERENCES users (id),
    status             TEXT NOT NULL DEFAULT 'READY' CHECK (status IN ('READY', 'LIVE', 'ENDED')),
    title              TEXT NOT NULL,
    category           TEXT NOT NULL,
    tags               TEXT[] NOT NULL DEFAULT '{}' CHECK (cardinality(tags) <= 3),
    record             BOOLEAN NOT NULL DEFAULT true,
    pip_corner         TEXT NOT NULL DEFAULT 'bottom_right'
                       CHECK (pip_corner IN ('top_left', 'top_right', 'bottom_left', 'bottom_right')),
    publisher_identity TEXT,
    started_at         TIMESTAMPTZ,
    ended_at           TIMESTAMPTZ,
    disconnected_at    TIMESTAMPTZ,
    peak_viewers       INTEGER NOT NULL DEFAULT 0,
    thumbnail_path     TEXT,
    hidden_at          TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 상태와 시각이 어긋나지 않게(READY → LIVE → ENDED, ENDED는 LIVE를 거친다)
    CHECK (
        (status = 'READY' AND started_at IS NULL AND ended_at IS NULL)
        OR (status = 'LIVE' AND started_at IS NOT NULL AND ended_at IS NULL)
        OR (status = 'ENDED' AND started_at IS NOT NULL AND ended_at IS NOT NULL)
    ),
    CHECK (hidden_at IS NULL OR status = 'ENDED'),
    UNIQUE (id, owner_id) -- chats의 (broadcast_id, streamer_id) 복합 FK 대상
);
-- 사용자당 동시 LIVE 1개. LIVE 목록 조회도 이 부분 인덱스를 쓴다.
CREATE UNIQUE INDEX broadcasts_one_live_per_owner ON broadcasts (owner_id) WHERE status = 'LIVE';
CREATE INDEX broadcasts_owner_idx ON broadcasts (owner_id, id);

-- ===== 채팅·모더레이션 =====
CREATE TABLE chats (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    broadcast_id BIGINT NOT NULL,
    streamer_id  BIGINT NOT NULL,
    user_id      BIGINT NOT NULL REFERENCES users (id),
    body         TEXT NOT NULL, -- 원문. 욕설 가림은 응답 시
    edited_at    TIMESTAMPTZ,
    deleted_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- streamer_id가 방송 소유자와 항상 같도록(차단·"이 시청자의 채팅 기록" 권한이 이 값에 의존)
    FOREIGN KEY (broadcast_id, streamer_id) REFERENCES broadcasts (id, owner_id)
);
CREATE INDEX chats_broadcast_idx ON chats (broadcast_id, id);
CREATE INDEX chats_streamer_user_idx ON chats (streamer_id, user_id, id);

-- 현재 차단 상태(스트리머·사용자당 1행). until NULL = 영구 밴, 지난 until = 풀린 임시차단.
CREATE TABLE bans (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    streamer_id BIGINT NOT NULL REFERENCES users (id),
    user_id     BIGINT NOT NULL REFERENCES users (id),
    until       TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (streamer_id, user_id),
    CHECK (streamer_id <> user_id)
);

CREATE TABLE ban_events (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    streamer_id BIGINT NOT NULL REFERENCES users (id),
    user_id     BIGINT NOT NULL REFERENCES users (id),
    action      TEXT NOT NULL CHECK (action IN ('timeout', 'ban', 'unban')),
    until       TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER ban_events_append_only BEFORE UPDATE OR DELETE OR TRUNCATE ON ban_events
    FOR EACH STATEMENT EXECUTE FUNCTION reject_modification();

-- ===== 팔로우 =====
CREATE TABLE follows (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    follower_id BIGINT NOT NULL REFERENCES users (id),
    streamer_id BIGINT NOT NULL REFERENCES users (id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (follower_id, streamer_id),
    CHECK (follower_id <> streamer_id)
);
CREATE INDEX follows_streamer_idx ON follows (streamer_id);

-- ===== 지갑·결제·후원 =====
CREATE TABLE wallets (
    user_id BIGINT PRIMARY KEY REFERENCES users (id),
    balance BIGINT NOT NULL DEFAULT 0 CHECK (balance BETWEEN 0 AND 3000000)
);

CREATE TABLE payment_orders (
    id                 BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    order_id           TEXT NOT NULL UNIQUE,
    payment_key        TEXT UNIQUE,
    user_id            BIGINT NOT NULL REFERENCES users (id),
    purpose            TEXT NOT NULL CHECK (purpose IN ('charge', 'donation')),
    amount             BIGINT NOT NULL CHECK (amount BETWEEN 1000 AND 1000000), -- 결제(=충전) 금액
    -- 바로 결제 후원: 결제창을 다녀오는 동안 후원 내용을 주문에 보관
    broadcast_id       BIGINT REFERENCES broadcasts (id),
    donation_amount    BIGINT CHECK (donation_amount BETWEEN 1000 AND 1000000),
    donation_anonymous BOOLEAN,
    donation_message   TEXT,
    status             TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'done', 'expired', 'failed')),
    expires_at         TIMESTAMPTZ NOT NULL,
    confirmed_at       TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (purpose = 'charge' AND broadcast_id IS NULL AND donation_amount IS NULL AND donation_anonymous IS NULL)
        OR (purpose = 'donation' AND broadcast_id IS NOT NULL AND donation_amount IS NOT NULL AND donation_anonymous IS NOT NULL)
    ),
    CHECK (status <> 'done' OR (payment_key IS NOT NULL AND confirmed_at IS NOT NULL))
);
CREATE INDEX payment_orders_pending_expiry_idx ON payment_orders (expires_at) WHERE status = 'pending';

-- 원장: 추가만. 참조는 유형별 FK. 후원 기록은 donations.ledger_entry_id가 이 행을 가리킨다.
CREATE TABLE ledger_entries (
    id               BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id          BIGINT NOT NULL REFERENCES users (id),
    type             TEXT NOT NULL CHECK (type IN ('charge', 'donation', 'clutch', 'burst', 'withdraw_forfeit')),
    amount           BIGINT NOT NULL CHECK (amount <> 0),
    balance_after    BIGINT NOT NULL CHECK (balance_after BETWEEN 0 AND 3000000),
    -- 주문 1건당 충전 1건: 결제 확정이 중복돼도 두 번 적립되지 않는다
    payment_order_id BIGINT UNIQUE REFERENCES payment_orders (id),
    broadcast_id     BIGINT REFERENCES broadcasts (id),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((type = 'charge') = (amount > 0)),
    CHECK ((type = 'charge') = (payment_order_id IS NOT NULL)),
    CHECK ((type IN ('donation', 'clutch', 'burst')) = (broadcast_id IS NOT NULL))
);
CREATE INDEX ledger_entries_user_idx ON ledger_entries (user_id, id);
CREATE TRIGGER ledger_entries_append_only BEFORE UPDATE OR DELETE OR TRUNCATE ON ledger_entries
    FOR EACH STATEMENT EXECUTE FUNCTION reject_modification();

-- 스트리머가 받은 비트(후원·유료 클러치·SUPER BURST). 받은 합계는 여기서 집계한다.
CREATE TABLE donations (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    broadcast_id    BIGINT NOT NULL REFERENCES broadcasts (id),
    donor_id        BIGINT NOT NULL REFERENCES users (id),
    kind            TEXT NOT NULL CHECK (kind IN ('donation', 'clutch', 'burst')),
    amount          BIGINT NOT NULL CHECK (amount > 0),
    anonymous       BOOLEAN NOT NULL DEFAULT false,
    message         TEXT,
    ledger_entry_id BIGINT NOT NULL UNIQUE REFERENCES ledger_entries (id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (kind <> 'donation' OR amount BETWEEN 1000 AND 1000000)
);
CREATE INDEX donations_broadcast_idx ON donations (broadcast_id, id);

-- ===== Clutch =====
-- 게이지 점수 이벤트. source: free/paid(클러치 버튼), burst(SUPER BURST), donation(후원 비트)
CREATE TABLE clutch_events (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    broadcast_id BIGINT NOT NULL REFERENCES broadcasts (id),
    user_id      BIGINT NOT NULL REFERENCES users (id),
    source       TEXT NOT NULL CHECK (source IN ('free', 'paid', 'burst', 'donation')),
    points       BIGINT NOT NULL CHECK (points > 0),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX clutch_events_gauge_idx ON clutch_events (broadcast_id, created_at);
CREATE INDEX clutch_events_combo_idx ON clutch_events (user_id, broadcast_id, created_at);

CREATE TABLE super_clutches (
    id             BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    broadcast_id   BIGINT NOT NULL REFERENCES broadcasts (id),
    triggered_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at     TIMESTAMPTZ NOT NULL,
    rotation_order BIGINT NOT NULL,
    CHECK (expires_at > triggered_at)
);
-- 쿨다운(마지막 발동 후 30분)·게이지 초기화 시점 조회
CREATE INDEX super_clutches_broadcast_idx ON super_clutches (broadcast_id, triggered_at);

-- ===== 녹화 =====
-- 방송과 1:1. recording(Egress 진행·파일 처리 중) → ready | failed, 보관 정리·방송 숨김 시 deleted.
CREATE TABLE recordings (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    broadcast_id BIGINT NOT NULL UNIQUE REFERENCES broadcasts (id),
    egress_id    TEXT UNIQUE,
    status       TEXT NOT NULL DEFAULT 'recording' CHECK (status IN ('recording', 'ready', 'failed', 'deleted')),
    object_key   TEXT,
    size_bytes   BIGINT,
    duration_sec INTEGER,
    expires_at   TIMESTAMPTZ,
    views        BIGINT NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (status <> 'ready' OR (object_key IS NOT NULL AND size_bytes IS NOT NULL
                                 AND duration_sec IS NOT NULL AND expires_at IS NOT NULL))
);
-- 최신순 목록(역방향 스캔)과 오래된 것부터 보관 정리 둘 다
CREATE INDEX recordings_ready_idx ON recordings (created_at) WHERE status = 'ready';

-- 조회수: 같은 시청자(u:<사용자id> / g:<익명id>) 하루(KST) 1회
CREATE TABLE recording_views (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    recording_id BIGINT NOT NULL REFERENCES recordings (id),
    viewer_key   TEXT NOT NULL,
    date_kst     DATE NOT NULL DEFAULT (now() AT TIME ZONE 'Asia/Seoul')::date,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (recording_id, viewer_key, date_kst)
);
