CREATE TABLE IF NOT EXISTS vault (
    id INT NOT NULL AUTO_INCREMENT,
    name varchar(100) NOT NULL,
    PRIMARY KEY (id),
    KEY idx_name (name)
);
