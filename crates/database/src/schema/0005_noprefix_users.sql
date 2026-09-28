-- Per-guild noprefix user whitelist
-- Only users in this table (+ guild owner + bot owner) can use noprefix commands
CREATE TABLE IF NOT EXISTS noprefix_users (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    added_by BIGINT NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (guild_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_noprefix_users_guild ON noprefix_users(guild_id);
