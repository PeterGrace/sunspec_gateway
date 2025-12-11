-- Add migration script here
CREATE TABLE IF NOT EXISTS bitfield_history (
    uniqueid VARCHAR(255) NOT NULL,
    field_name VARCHAR(255) NOT NULL,
    UNIQUE(uniqueid, field_name)
    );
