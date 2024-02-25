-- Add migration script here
CREATE TABLE IF NOT EXISTS "devices"
(
    "id"                VARCHAR(255) PRIMARY KEY NOT NULL,
    "mac"               MACADDR NOT NULL,
    "broadcast_addr"    VARCHAR(39) NOT NULL,
    "ip"                INET NOT NULL,
    "times"             BIGINT[]
)
