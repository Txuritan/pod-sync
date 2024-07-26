CREATE TABLE IF NOT EXISTS subscription_guids (
    subscription_id INTEGER NOT NULL,
    guid TEXT NOT NULL UNIQUE,
    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP, -- TODO: add index
    PRIMARY KEY (subscription_id, guid),
    FOREIGN KEY (subscription_id) REFERENCES subscriptions (id) ON UPDATE CASCADE ON DELETE CASCADE
);
