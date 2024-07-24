CREATE TABLE IF NOT EXISTS user_sessions (
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL,
    expires INTEGER NOT NULL,
    PRIMARY KEY (user_id, token)
);
