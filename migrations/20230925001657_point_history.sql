-- Add migration script here
CREATE TABLE IF NOT EXISTS point_history
(
    uniqueid VARCHAR(256)  NOT NULL,
    timestamp INTEGER  NOT NULL,
    value VARCHAR(256) NOT NULL,
    PRIMARY KEY (uniqueid,timestamp)
);
