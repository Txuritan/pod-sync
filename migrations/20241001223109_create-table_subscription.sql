CREATE TABLE subscription (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    user_id INTEGER NOT NULL,
    podcast_id INTEGER NOT NULL,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES user (id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (podcast_id) REFERENCES podcast (id) ON UPDATE CASCADE ON DELETE CASCADE
);
