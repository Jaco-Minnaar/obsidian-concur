ALTER TABLE file ADD hash TEXT NOT NULL DEFAULT '';

CREATE INDEX idx_hash 
ON file (hash);
