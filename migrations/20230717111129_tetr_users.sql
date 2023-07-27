-- Add migration script here

CREATE TABLE osu_users (
    user_id     BIGINT   NOT NULL PRIMARY KEY,
    id          BIGINT   NOT NULL UNIQUE,
    last_update DATETIME NOT NULL,
    tr          REAL     NULL,
    apm         REAL     NULL,
    pps         REAL     NULL,
    vs          REAL     NULL,
    sprint      REAL     NULL,
    blitz       REAL     NULL
);