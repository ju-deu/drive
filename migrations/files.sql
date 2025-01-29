
CREATE TABLE file (
    reference_uuid VARCHAR PRIMARY KEY,
    owner_uuid VARCHAR NOT NULL,
    filename VARCHAR NOT NULL,

    timestamp bigint DEFAULT EXTRACT(EPOCH FROM NOW())
);