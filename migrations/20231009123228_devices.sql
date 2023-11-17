-- Add migration script here
CREATE TABLE IF NOT EXISTS "devices"
(
    "id"                VARCHAR(255) PRIMARY KEY NOT NULL,
    "mac"               VARCHAR(17) NOT NULL,
    "broadcast_addr"    VARCHAR(39) NOT NULL,
    "ip"                VARCHAR(39) NOT NULL,
    "times"             BIGINT[]
)
