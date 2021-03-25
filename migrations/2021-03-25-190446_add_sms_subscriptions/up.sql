-- Your SQL goes here
CREATE TABLE sms_subscriptions(
    id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    phone_number Varchar(256) NOT NULL,
    confirmed BOOL NOT NULL DEFAULT false
);
