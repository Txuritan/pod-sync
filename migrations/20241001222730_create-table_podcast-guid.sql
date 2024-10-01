CREATE TABLE podcast_guid (
    podcast_id INTEGER NOT NULL,

    feed_guid TEXT NOT NULL,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP,

    PRIMARY KEY (podcast_id, feed_guid),
    FOREIGN KEY (podcast_id) REFERENCES podcast (id) ON UPDATE CASCADE ON DELETE CASCADE
);
