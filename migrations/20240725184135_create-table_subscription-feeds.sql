CREATE TABLE IF NOT EXISTS subscription_feeds (
    subscription_id INTEGER NOT NULL,
    feed TEXT NOT NULL UNIQUE,
    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP, -- TODO: add index
    PRIMARY KEY (subscription_id, feed),
    FOREIGN KEY (subscription_id) REFERENCES subscription (id) ON UPDATE CASCADE ON DELETE CASCADE
);
