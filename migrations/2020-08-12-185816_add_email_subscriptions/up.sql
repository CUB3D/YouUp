-- Your SQL goes here
CREATE TABLE email_subscriptions(
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    email Varchar(256) NOT NULL,
    confirmed BOOL NOT NULL DEFAULT false
);
