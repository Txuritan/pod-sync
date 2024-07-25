CREATE TABLE IF NOT EXISTS task_deletion (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    subscription_id INTEGER NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP, -- TODO: add index
    UNIQUE (user_id, subscription_id),
    FOREIGN KEY (user_id) REFERENCES user (id) ON UPDATE CASCADE ON DELETE CASCADE
    FOREIGN KEY (subscription_id) REFERENCES subscription (id) ON UPDATE CASCADE ON DELETE CASCADE
);
