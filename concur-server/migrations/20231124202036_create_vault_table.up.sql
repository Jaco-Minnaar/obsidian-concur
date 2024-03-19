CREATE TABLE vault (
    id              INTEGER     PRIMARY KEY,
    vault_name      TEXT        NOT NULL
);

CREATE INDEX idx_name on vault (vault_name);
