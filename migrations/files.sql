
CREATE TABLE IF NOT EXISTS file (
    reference_uuid VARCHAR PRIMARY KEY,
    owner_uuid VARCHAR NOT NULL,
    filename VARCHAR NOT NULL,

    relative_path VARCHAR,
    absolute_path VARCHAR,
    size BIGINT,

    timestamp bigint DEFAULT EXTRACT(EPOCH FROM NOW())
);