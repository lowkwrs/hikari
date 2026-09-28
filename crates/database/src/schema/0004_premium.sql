CREATE TABLE IF NOT EXISTS premium_users (
    user_id BIGINT PRIMARY KEY,
    activated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    activated_by BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS premium_guilds (
    guild_id BIGINT PRIMARY KEY,
    activated_by BIGINT NOT NULL,
    activated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    noprefix_enabled BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_premium_guilds_activated_by ON premium_guilds(activated_by);
