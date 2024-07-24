CREATE TABLE IF NOT EXISTS subscriptions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,
    podcast TEXT NOT NULL,
    action TEXT NOT NULL,
    timestamp TEXT NOT NULL
);
