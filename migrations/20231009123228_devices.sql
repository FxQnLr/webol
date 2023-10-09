-- Add migration script here
CREATE TABLE "devices"
(
    "id"                TEXT PRIMARY KEY NOT NULL,
    "mac"               TEXT NOT NULL,
    "broadcast_addr"    TEXT NOT NULL
)