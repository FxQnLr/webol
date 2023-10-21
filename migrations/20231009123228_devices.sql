-- Add migration script here
CREATE TABLE IF NOT EXISTS "devices"
(
    "id"                TEXT PRIMARY KEY NOT NULL,
    "mac"               TEXT NOT NULL,
    "broadcast_addr"    TEXT NOT NULL
)
