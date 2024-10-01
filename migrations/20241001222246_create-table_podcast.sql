CREATE TABLE podcast (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    title TEXT NOT NULL,

    description TEXT,
    image TEXT,
    language TEXT,
    link TEXT,
    copyright TEXT,

    itunes_author TEXT,
    itunes_category TEXT,
    itunes_subcategory TEXT,
    itunes_owner_name TEXT,
    itunes_owner_email TEXT,
    itunes_type TEXT,
    itunes_summary TEXT,

    created TIMESTAMP NOT NULL DEFAULT (DATETIME('now')),
    updated TIMESTAMP,
    deleted TIMESTAMP
);
