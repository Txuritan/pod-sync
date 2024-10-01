CREATE TABLE podcast_episode (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    podcast_id INTEGER NOT NULL,

    title TEXT,
    enclosure TEXT,
    guid TEXT,
    published TEXT,
    description TEXT,

    itunes_season INTEGER,
    itunes_episode INTEGER,
    itunes_duration TEXT,
    itunes_image TEXT,
    itunes_type TEXT,
    itunes_subtitle TEXT,
    itunes_summary TEXT,

    podcast_transcript TEXT,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP,

    FOREIGN KEY (podcast_id) REFERENCES podcast (id) ON UPDATE CASCADE ON DELETE CASCADE
);
