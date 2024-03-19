CREATE TABLE IF NOT EXISTS file (
    id          INTEGER     PRIMARY KEY,
    path        TEXT        NOT NULL,
    content     TEXT        NOT NULL,
    vault_id    INTEGER     NOT NULL,
    FOREIGN KEY (vault_id) 
        REFERENCES vault (id)
);

CREATE INDEX idx_path on file (path);
