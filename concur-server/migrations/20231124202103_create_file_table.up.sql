CREATE TABLE IF NOT EXISTS file (
    id          INTEGER     PRIMARY KEY,
    path        TEXT        NOT NULL,
    content     TEXT        NOT NULL,
    last_sync   INTEGER     NOT NULL,
    vault_id    INTEGER     NOT NULL,
    FOREIGN KEY (vault_id) 
        REFERENCES vault (id)
);

CREATE INDEX idx_path on file (path);

CREATE UNIQUE INDEX idx_path_vault_id 
ON file (path, vault_id);

CREATE INDEX idx_last_sync ON file (last_sync);
