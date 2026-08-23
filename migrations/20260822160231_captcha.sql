-- Add migration script here
CREATE TABLE IF NOT EXISTS captcha (
    guild_id BIGINT NOT NULL PRIMARY KEY,
    role_id BIGINT NOT NULL
);