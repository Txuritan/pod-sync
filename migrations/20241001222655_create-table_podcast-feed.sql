CREATE TABLE podcast_feed (
    podcast_id INTEGER NOT NULL,

    feed_url TEXT NOT NULL,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP,

    PRIMARY KEY (podcast_id, feed_url),
    FOREIGN KEY (podcast_id) REFERENCES podcast (id) ON UPDATE CASCADE ON DELETE CASCADE
);
