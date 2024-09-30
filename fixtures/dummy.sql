INSERT INTO
    users (id, username, email, password_hash)
VALUES
    (56631, 'example', 'example@example.com', '');

INSERT INTO
    user_sessions (user_id, token, expires)
VALUES
    (56631, 'asdfghjklqwertyuiopzxcvbnm', (datetime('now', "+21 days")));



INSERT INTO
    subscriptions (id)
VALUES
    (80890),
    (92766),
    (37239);

INSERT INTO
    user_subscriptions (user_id, subscription_id)
VALUES
    (56631, 80890),
    (56631, 92766),
    (56631, 37239);

INSERT INTO
    subscription_feeds (subscription_id, feed, created)
VALUES
    (80890, 'http://one.example.com/feed.rss', (DATETIME('now', '-7 days'))),
    (92766, 'http://two-old.example.com/feed.rss', (DATETIME('now', '-7 days'))),
    (92766, 'http://two-new.example.com/feed.rss', (DATETIME('now'))),
    (37239, 'http://three.example.com/feed.rss', (DATETIME('now', '-7 days')));

INSERT INTO
    subscription_guids (subscription_id, guid, created)
VALUES
    (80890, X'1c736505c5e05b9d94cddcb383069b49', (DATETIME('now', '-7 days'))), -- 1c736505-c5e0-5b9d-94cd-dcb383069b49
    (92766, X'7f3f76e479d15a058d215438d032fdd6', (DATETIME('now', '-7 days'))), -- 7f3f76e4-79d1-5a05-8d21-5438d032fdd6
    (37239, X'cbfab27c55295fe2a7e1607bdb128145', (DATETIME('now', '-7 days'))), -- cbfab27c-5529-5fe2-a7e1-607bdb128145
    (37239, X'8056e44f978e44b3b34ae99c79b6d891', (DATETIME('now'))); -- 8056e44f-978e-44b3-b34a-e99c79b6d891



INSERT INTO
    task_deletions (id, user_id, subscription_id, status)
VALUES
    (1, 56631, 80890, 'pending'),
    (2, 56631, 92766, 'success'),
    (3, 56631, 37239, 'failure');
