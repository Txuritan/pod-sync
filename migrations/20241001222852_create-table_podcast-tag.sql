CREATE TABLE podcast_tag (
    podcast_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP,

    PRIMARY KEY (podcast_id, tag_id),
    FOREIGN KEY (podcast_id) REFERENCES podcast (id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tag (id) ON UPDATE CASCADE ON DELETE CASCADE
);
