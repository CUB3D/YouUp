CREATE TABLE incident_status_update (
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    status_type INTEGER NOT NULL,
    message Varchar(256) NOT NULL,
    incident INTEGER NOT NULL,
    FOREIGN KEY (status_type) REFERENCES incident_status_type(id),
    FOREIGN KEY (incident) REFERENCES incidents(id)
)
