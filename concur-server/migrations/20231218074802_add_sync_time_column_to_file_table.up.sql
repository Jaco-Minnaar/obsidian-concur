ALTER TABLE file ADD last_sync INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_last_sync ON file (last_sync);
