CREATE TABLE IF NOT EXISTS subscriptions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP
);
