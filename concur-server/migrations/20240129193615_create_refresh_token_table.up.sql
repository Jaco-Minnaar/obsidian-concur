CREATE TABLE IF NOT EXISTS refresh_token (
    id INT NOT NULL AUTO_INCREMENT,
    user_id INT NOT NULL,
    client_id INT NOT NULL,
    token varchar(255) NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    KEY idx_user_id (user_id),
    KEY idx_client_id (client_id)
);
