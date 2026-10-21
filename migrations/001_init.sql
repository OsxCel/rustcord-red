CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_sessions_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS stories (
    id UUID PRIMARY KEY,
    author VARCHAR(255) NOT NULL,
    caption TEXT NOT NULL,
    media_type VARCHAR(50) NOT NULL,
    media_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS search_history (
    id UUID PRIMARY KEY,
    scope VARCHAR(50) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    entity_label VARCHAR(255) NOT NULL,
    query TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL,
    duration_ms BIGINT NOT NULL,
    result_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS call_sessions (
    id UUID PRIMARY KEY,
    room_id VARCHAR(255) NOT NULL,
    call_type VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_ms BIGINT NOT NULL DEFAULT 0,
    participants JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dm_messages (
    id UUID PRIMARY KEY,
    from_user VARCHAR(255) NOT NULL,
    to_user VARCHAR(255) NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    owner VARCHAR(255) NOT NULL,
    members JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS voice_rooms (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    participants JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reels (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    author VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    duration VARCHAR(20) NOT NULL,
    views BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_flag BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions (token);
CREATE INDEX IF NOT EXISTS idx_stories_created_at ON stories (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_history_created_at ON search_history (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_call_sessions_room_id ON call_sessions (room_id);
CREATE INDEX IF NOT EXISTS idx_dm_messages_users ON dm_messages (from_user, to_user);
CREATE INDEX IF NOT EXISTS idx_groups_owner ON groups (owner);
CREATE INDEX IF NOT EXISTS idx_voice_rooms_status ON voice_rooms (status);
CREATE INDEX IF NOT EXISTS idx_reels_created_at ON reels (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications (created_at DESC);
