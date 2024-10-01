CREATE TABLE account (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    user_id INTEGER NOT NULL,

    kind TEXT NOT NULL,
    email TEXT NOT NULL,
    password TEXT,
    external_id TEXT,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES user (id) ON UPDATE CASCADE ON DELETE CASCADE
);
