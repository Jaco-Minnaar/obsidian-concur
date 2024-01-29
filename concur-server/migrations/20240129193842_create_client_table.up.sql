CREATE TABLE IF NOT EXISTS client (
    id INT NOT NULL AUTO_INCREMENT,
    user_id INT NOT NULL,
    name varchar(100) NOT NULL,
    PRIMARY KEY (id),
    KEY idx_user_id (user_id) 
);
