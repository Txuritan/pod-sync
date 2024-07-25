CREATE TABLE IF NOT EXISTS episode_actions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    device_id INTEGER NOT NULL,
    podcast TEXT NOT NULL,
    episode TEXT NOT NULL,
    action TEXT NOT NULL,
    position INTEGER,
    started INTEGER,
    total INTEGER,
    timestamp INTEGER NOT NULL
);
