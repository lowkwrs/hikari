CREATE TABLE IF NOT EXISTS antinuke_guild_config (
    guild_id        BIGINT PRIMARY KEY,
    enabled         BOOLEAN NOT NULL DEFAULT FALSE,
    log_channel_id  BIGINT,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS antinuke_module_config (
    guild_id        BIGINT NOT NULL,
    action_type     TEXT NOT NULL,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    threshold       INTEGER NOT NULL,
    window_secs     INTEGER NOT NULL,
    punishment      TEXT NOT NULL,
    log_only        BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (guild_id, action_type),
    FOREIGN KEY (guild_id) REFERENCES antinuke_guild_config(guild_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS antinuke_whitelist (
    guild_id        BIGINT NOT NULL,
    user_id         BIGINT NOT NULL,
    added_by        BIGINT NOT NULL,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, user_id)
);

CREATE TABLE IF NOT EXISTS antinuke_incident_log (
    id              BIGSERIAL PRIMARY KEY,
    guild_id        BIGINT NOT NULL,
    user_id         BIGINT NOT NULL,
    action_type     TEXT NOT NULL,
    score           INTEGER NOT NULL,
    punishment      TEXT NOT NULL,
    count_in_window INTEGER NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_antinuke_incident_guild_time
    ON antinuke_incident_log (guild_id, created_at DESC);
