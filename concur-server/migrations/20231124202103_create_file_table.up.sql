CREATE TABLE IF NOT EXISTS file (
    id INT NOT NULL AUTO_INCREMENT,
    path VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    vault_id INT NOT NULL,
    PRIMARY KEY (id),
    KEY idx_vault_id (vault_id),
    KEY idx_path (path)
);
