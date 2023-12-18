ALTER TABLE file ADD last_sync DATETIME NOT NULL;
ALTER TABLE file ADD INDEX idx_last_sync (last_sync);
