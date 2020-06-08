-- Create projects
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    url VARCHAR(256) NOT NULL,
    name VARCHAR(32) UNIQUE NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL
);

-- Create status
-- time is how long the request takes in ms
CREATE TABLE status (
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    project INTEGER NOT NULL,
    time INTEGER NOT NULL,
    status_code INTEGER NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    FOREIGN KEY (project) REFERENCES projects(id)
);
