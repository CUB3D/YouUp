CREATE TABLE incidents (
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    status Varchar(32) NOT NULL,
    message Varchar(256) NOT NULL,
    project INTEGER NOT NULL,
    FOREIGN KEY (project) REFERENCES projects(id)
);