-- Add migration script here

CREATE TABLE tetr_users (
    user_id     BIGINT   NOT NULL PRIMARY KEY,
    id          TEXT     NOT NULL UNIQUE,
    last_update DATETIME NOT NULL,
    tr          REAL     NOT NULL,
    rank        TEXT     NOT NULL,
    apm         REAL     NULL,
    pps         REAL     NULL,
    vs          REAL     NULL,
    sprint      REAL     NULL,
    blitz       REAL     NULL
);