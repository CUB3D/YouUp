-- Your SQL goes here-- Your SQL goes here
CREATE TABLE webhook_subscriptions(
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    url Varchar(256) NOT NULL,
    enabled BOOL NOT NULL DEFAULT true
);
